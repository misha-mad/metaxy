use quote::quote;
use syn::{FnArg, ItemFn, ReturnType, Type};

use crate::attrs::HandlerAttrs;
use crate::codegen::{is_headers_type, is_ref_type};

/// Checks whether a type syntactically ends with `StreamSender`.
fn is_stream_sender_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "StreamSender";
    }
    false
}

/// Transforms a user-defined async function into a streaming Vercel handler
/// using Axum + `stream_response`.
///
/// Unlike query/mutation handlers that use `service_fn`, streaming handlers
/// produce an Axum-based binary with `VercelLayer`.
#[expect(
    clippy::needless_pass_by_value,
    reason = "ItemFn is owned from parse_macro_input"
)]
pub(crate) fn build_stream_handler(
    func: ItemFn,
    attrs: HandlerAttrs,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let HandlerAttrs {
        cache_config: _,
        init_fn,
        timeout_secs,
        idempotent: _,
    } = attrs;

    if func.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            func.sig.fn_token,
            "RPC stream handlers must be async functions",
        ));
    }

    if !matches!(func.sig.output, ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            &func.sig.output,
            "RPC stream handlers must not have a return type (use tx.send() instead)",
        ));
    }

    let fn_name = &func.sig.ident;
    let fn_block = &func.block;

    // Separate typed parameters into input, headers, state, and stream sender.
    let typed_params: Vec<_> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat) => Some(pat),
            _ => None,
        })
        .collect();

    let mut input_param = None;
    let mut headers_param = None;
    let mut state_param = None;
    let mut sender_param = None;

    for param in &typed_params {
        if is_stream_sender_type(&param.ty) {
            if sender_param.is_some() {
                return Err(syn::Error::new_spanned(
                    &func.sig.inputs,
                    "RPC stream handlers accept at most one StreamSender parameter",
                ));
            }
            sender_param = Some(*param);
        } else if is_headers_type(&param.ty) {
            if headers_param.is_some() {
                return Err(syn::Error::new_spanned(
                    &func.sig.inputs,
                    "RPC handlers accept at most one Headers parameter",
                ));
            }
            headers_param = Some(*param);
        } else if let Type::Reference(r) = &*param.ty {
            if r.mutability.is_some() {
                return Err(syn::Error::new_spanned(
                    &param.ty,
                    "state parameter must be a shared reference (&T), not &mut T",
                ));
            }
            if state_param.is_some() {
                return Err(syn::Error::new_spanned(
                    &func.sig.inputs,
                    "RPC handlers accept at most one state parameter",
                ));
            }
            state_param = Some(*param);
        } else if input_param.is_some() {
            return Err(syn::Error::new_spanned(
                &func.sig.inputs,
                "RPC handlers accept at most one input parameter",
            ));
        } else {
            input_param = Some(*param);
        }
    }

    if sender_param.is_none() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "RPC stream handlers must accept a StreamSender parameter",
        ));
    }

    if state_param.is_some() && init_fn.is_none() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "state parameter requires init = \"...\" attribute",
        ));
    }

    let input_type = if let Some(param) = input_param {
        let ty = &param.ty;
        quote! { #ty }
    } else {
        quote! { () }
    };

    // Build inner function parameters preserving original order.
    let inner_fn_params: Vec<_> = typed_params
        .iter()
        .map(|param| {
            let pat = &param.pat;
            let ty = &param.ty;
            quote! { #pat: #ty }
        })
        .collect();

    let call_args: Vec<_> = typed_params
        .iter()
        .map(|param| {
            if is_stream_sender_type(&param.ty) {
                quote! { __tx }
            } else if is_headers_type(&param.ty) {
                quote! { __headers }
            } else if is_ref_type(&param.ty) {
                quote! { __state }
            } else {
                quote! { __input }
            }
        })
        .collect();

    // Generate OnceLock static when init returns state.
    let state_static = if let Some(sp) = state_param {
        let Type::Reference(r) = &*sp.ty else {
            unreachable!("state_param is always a reference");
        };
        let inner_ty = &r.elem;
        quote! {
            static __RPC_STATE: std::sync::OnceLock<#inner_ty> = std::sync::OnceLock::new();
        }
    } else {
        quote! {}
    };

    // Generate init call inside main's block_on.
    let init_call = if let Some(ref path) = init_fn {
        let path_ident: proc_macro2::TokenStream = path.parse().map_err(|_| {
            syn::Error::new_spanned(&func.sig, format!("invalid init function path: `{path}`"))
        })?;
        if state_param.is_some() {
            quote! {
                __RPC_STATE.set(#path_ident().await).expect("BUG: OnceLock already set");
            }
        } else {
            quote! {
                #path_ident().await;
            }
        }
    } else {
        quote! {}
    };

    let extract_state = if state_param.is_some() {
        quote! { let __state = __RPC_STATE.get().expect("BUG: init not called"); }
    } else {
        quote! {}
    };

    // Determine how to extract input and headers from the axum handler.
    let handler_params;
    let extract_input;
    let extract_headers;

    let has_input = input_param.is_some();
    let has_headers = headers_param.is_some();

    match (has_input, has_headers) {
        (true, true) => {
            handler_params = quote! {
                __header_map: ::metaxy::__private::axum::http::HeaderMap,
                ::metaxy::__private::axum::Json(__input): ::metaxy::__private::axum::Json<#input_type>,
            };
            extract_input = quote! {};
            extract_headers = quote! { let __headers = __header_map; };
        }
        (true, false) => {
            handler_params = quote! {
                ::metaxy::__private::axum::Json(__input): ::metaxy::__private::axum::Json<#input_type>,
            };
            extract_input = quote! {};
            extract_headers = quote! {};
        }
        (false, true) => {
            handler_params = quote! {
                __header_map: ::metaxy::__private::axum::http::HeaderMap,
            };
            extract_input = quote! { let __input: () = (); };
            extract_headers = quote! { let __headers = __header_map; };
        }
        (false, false) => {
            handler_params = quote! {};
            extract_input = quote! { let __input: () = (); };
            extract_headers = quote! {};
        }
    }

    let timeout_wrapper = if let Some(secs) = timeout_secs {
        quote! {
            let __timeout_dur = ::std::time::Duration::from_secs(#secs);
            let __deadline = ::metaxy::__private::tokio::time::Instant::now() + __timeout_dur;
        }
    } else {
        quote! {}
    };

    let create_sender = if timeout_secs.is_some() {
        quote! {
            let __timeout_tx = __raw_tx.clone();
            let __tx = ::metaxy::StreamSender::new(__raw_tx);
        }
    } else {
        quote! {
            let __tx = ::metaxy::StreamSender::new(__raw_tx);
        }
    };

    let invoke_user_fn = if timeout_secs.is_some() {
        quote! {
            match ::metaxy::__private::tokio::time::timeout_at(
                __deadline,
                #fn_name(#(#call_args),*),
            ).await {
                Ok(()) => {}
                Err(_) => {
                    let _ = __timeout_tx.send(Ok(
                        ::metaxy::__private::hyper::body::Bytes::from("event: error\ndata: \"Handler timed out\"\n\n")
                    )).await;
                }
            }
        }
    } else {
        quote! { #fn_name(#(#call_args),*).await; }
    };

    let expanded = quote! {
        #state_static

        #[::metaxy::__private::tokio::main]
        async fn main() -> Result<(), ::metaxy::__private::vercel_runtime::Error> {
            #init_call

            let __router = ::metaxy::__private::axum::Router::new()
                .fallback(::metaxy::__private::axum::routing::post(__rpc_stream_handler));

            let __app = ::metaxy::__private::tower::ServiceBuilder::new()
                .layer(::metaxy::__private::vercel_runtime::axum::VercelLayer::new())
                .service(__router);

            ::metaxy::__private::vercel_runtime::run(__app).await
        }

        async fn __rpc_stream_handler(
            #handler_params
        ) -> impl ::metaxy::__private::axum::response::IntoResponse {
            #extract_input
            #extract_headers
            #extract_state
            #timeout_wrapper

            ::metaxy::__private::vercel_runtime::axum::stream_response(move |__raw_tx| async move {
                #create_sender

                async fn #fn_name(#(#inner_fn_params),*)
                #fn_block

                #invoke_user_fn
            })
        }
    };

    Ok(expanded)
}
