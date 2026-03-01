#![warn(missing_docs)]
//! Procedural macros that turn async Rust functions into
//! Metaxy RPC handlers.
//!
//! Part of the [`metaxy`](https://github.com/misha-mad/metaxy) project
//! — end-to-end typesafe RPC between Rust lambdas on Vercel and any TypeScript frontend.
//!
//! # Quick Start
//!
//! Use the [`metaxy`](https://crates.io/crates/metaxy) facade crate
//! which re-exports these macros together with all runtime dependencies:
//!
//! ```toml
//! [dependencies]
//! metaxy = "0.1"
//! serde      = { version = "1", features = ["derive"] }
//! ```
//!
//! Then annotate an async function:
//!
//! ```rust,ignore
//! use metaxy::rpc_query;
//!
//! #[rpc_query]
//! async fn hello(name: String) -> String {
//!     format!("Hello, {}!", name)
//! }
//! ```
//!
//! This single attribute generates a complete Vercel-compatible binary with a
//! `main()` entry point — no extra boilerplate needed.
//!
//! # What Gets Generated
//!
//! Each macro invocation produces:
//!
//! - A `main()` entry point with a single-threaded tokio runtime that calls `vercel_runtime::run`.
//! - An `OPTIONS` handler that returns `204` with CORS headers.
//! - HTTP method validation (`GET` for queries, `POST` for mutations).
//! - Input deserialization — from the `?input=<JSON>` query parameter (queries)
//!   or from the JSON request body (mutations).
//! - Serialization of the return value into a JSON response.
//! - Automatic error responses when the function returns `Result::Err`.
//!
//! # Response Format
//!
//! **Success** (HTTP 200):
//! ```json
//! { "result": { "type": "response", "data": <value> } }
//! ```
//!
//! **Error** (HTTP 400):
//! ```json
//! { "error": { "type": "error", "message": "<description>" } }
//! ```
//!
//! # Supported Signatures
//!
//! | Signature | Input | Output |
//! |-----------|-------|--------|
//! | `async fn f() -> T` | none | `T` serialized |
//! | `async fn f(input: I) -> T` | `I` deserialized | `T` serialized |
//! | `async fn f() -> Result<T, E>` | none | `Ok` → 200, `Err` → 400 |
//! | `async fn f(input: I) -> Result<T, E>` | `I` deserialized | `Ok` → 200, `Err` → 400 |
//!
//! More than one parameter is a **compilation error**.
//!
//! # CORS
//!
//! Every response includes the following headers:
//!
//! - `Access-Control-Allow-Origin: *`
//! - `Access-Control-Allow-Methods: GET, POST, OPTIONS`
//! - `Access-Control-Allow-Headers: Content-Type`
//! - `Access-Control-Max-Age: 86400`
//!
//! # Companion Crate
//!
//! [`metaxy-cli`](https://crates.io/crates/metaxy-cli) scans your
//! `#[rpc_query]` / `#[rpc_mutation]` functions and generates TypeScript type
//! definitions and a fully typed RPC client for use in any TypeScript frontend.

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

mod attrs;
mod codegen;

use attrs::parse_handler_attrs;
use codegen::{HandlerKind, build_handler};

/// Generates a Vercel-compatible lambda handler from an async **query** function.
///
/// The annotated function becomes a **GET** endpoint. Input is read from the
/// `?input=<JSON>` query parameter, and the return value is serialized as JSON.
///
/// The macro generates:
/// - A `main()` entry point with a single-threaded tokio runtime that calls `vercel_runtime::run`.
/// - Automatic JSON deserialization of the `input` query parameter.
/// - Automatic JSON serialization of the return value.
/// - CORS headers on every response (including `OPTIONS` preflight → `204`).
/// - HTTP method validation — only `GET` is accepted; other methods return `405`.
/// - If the function returns `Result<T, E>`, `Err` is mapped to a `400` JSON
///   error response automatically.
///
/// # Examples
///
/// **No input:**
///
/// ```rust,ignore
/// use metaxy_macro::rpc_query;
///
/// #[rpc_query]
/// async fn version() -> String {
///     "1.0.0".to_string()
/// }
/// ```
///
/// **With input parameter:**
///
/// ```rust,ignore
/// use metaxy_macro::rpc_query;
///
/// #[rpc_query]
/// async fn hello(name: String) -> String {
///     format!("Hello, {}!", name)
/// }
/// ```
///
/// **Returning `Result`:**
///
/// ```rust,ignore
/// use metaxy_macro::rpc_query;
///
/// #[rpc_query]
/// async fn find_user(id: u32) -> Result<String, String> {
///     if id == 0 {
///         Err("not found".into())
///     } else {
///         Ok(format!("user_{}", id))
///     }
/// }
/// ```
///
/// **Returning a `Vec`:**
///
/// ```rust,ignore
/// use metaxy_macro::rpc_query;
///
/// #[rpc_query]
/// async fn list_tags() -> Vec<String> {
///     vec!["rust".into(), "vercel".into()]
/// }
/// ```
///
/// # Caching
///
/// Use the `cache` attribute to generate `Cache-Control` headers on successful
/// responses. On Vercel this enables edge caching without infrastructure changes.
///
/// ```rust,ignore
/// // CDN caches for 1 hour, browser always revalidates
/// #[rpc_query(cache = "1h")]
/// async fn get_settings() -> Settings { /* ... */ }
/// // → Cache-Control: public, max-age=0, s-maxage=3600
///
/// // CDN caches 5 min, serves stale up to 1 hour while revalidating
/// #[rpc_query(cache = "5m", stale = "1h")]
/// async fn get_feed() -> Vec<Post> { /* ... */ }
/// // → Cache-Control: public, max-age=0, s-maxage=300, stale-while-revalidate=3600
///
/// // Browser-only cache, no CDN
/// #[rpc_query(cache = "private, 10m")]
/// async fn get_profile() -> Profile { /* ... */ }
/// // → Cache-Control: private, max-age=600
/// ```
///
/// Duration shorthand: `30s`, `5m`, `1h`, `1d`. Error responses never receive
/// cache headers. Mutations (`#[rpc_mutation]`) do not support caching.
///
/// # Initialization
///
/// Use the `init` attribute to run an async function once at cold start.
/// The init function can optionally return the shared state injected as `&T`:
///
/// ```rust,ignore
/// // Side-effects only (logger, env loading)
/// #[rpc_query(init = "setup")]
/// async fn get_data() -> Data { /* ... */ }
///
/// // With state injection
/// #[rpc_query(init = "setup")]
/// async fn get_user(id: u32, state: &AppState) -> User {
///     state.pool.query("...").await
/// }
///
/// // Combined with cache
/// #[rpc_query(init = "setup", cache = "1h")]
/// async fn get_user(id: u32, state: &AppState) -> User { /* ... */ }
/// ```
///
/// The macro distinguishes state (`&T`) from input (`T`) by reference syntax.
/// A `&T` parameter requires `init`; `&mut T` is rejected.
///
/// # Timeout
///
/// Use the `timeout` attribute to enforce a per-procedure server-side timeout.
/// If the handler does not complete within the specified duration, the request
/// returns a `504` error response with `"Handler timed out"`.
///
/// ```rust,ignore
/// #[rpc_query(timeout = "30s")]
/// async fn slow_query() -> String { /* ... */ }
///
/// #[rpc_query(timeout = "5m", cache = "1h")]
/// async fn expensive(id: u32) -> Report { /* ... */ }
/// ```
///
/// Duration shorthand: `30s`, `5m`, `1h`, `1d`. The timeout is also forwarded
/// to the generated TypeScript client as a per-procedure default.
///
/// # Idempotent
///
/// The `idempotent` flag is **not** accepted on queries — queries are inherently
/// idempotent (GET requests). Using `#[rpc_query(idempotent)]` is a compiler error.
///
/// # Limitations
///
/// - `Result` and `Headers` are detected by **name only** (last path segment).
///   Type aliases like `type MyResult<T> = Result<T, MyError>` will not be
///   recognized, and custom types named `Result` or `Headers` will be falsely
///   matched. Use the canonical names directly.
///
/// # Compile errors
///
/// The macro rejects functions with more than one parameter:
///
/// ```rust,compile_fail,ignore
/// #[rpc_query]
/// async fn bad(a: String, b: u32) -> String { todo!() }
/// // error: RPC handlers accept at most one input parameter
/// ```
#[proc_macro_attribute]
pub fn rpc_query(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = match parse_handler_attrs(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };
    if attrs.idempotent {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "idempotent is only valid on mutations (queries are inherently idempotent)",
        )
        .to_compile_error()
        .into();
    }
    let input_fn = parse_macro_input!(item as ItemFn);
    build_handler(input_fn, HandlerKind::Query, attrs)
        .map(Into::into)
        .unwrap_or_else(|e| e.to_compile_error().into())
}

/// Generates a Vercel-compatible lambda handler from an async **mutation** function.
///
/// Works like [`rpc_query`] but creates a **POST** endpoint.
/// Input is read from the **JSON request body** instead of query parameters.
///
/// The macro generates:
/// - A `main()` entry point with a single-threaded tokio runtime that calls `vercel_runtime::run`.
/// - Automatic JSON deserialization of the request body.
/// - Automatic JSON serialization of the return value.
/// - CORS headers on every response (including `OPTIONS` preflight → `204`).
/// - HTTP method validation — only `POST` is accepted; other methods return `405`.
/// - If the function returns `Result<T, E>`, `Err` is mapped to a `400` JSON
///   error response automatically.
///
/// # Examples
///
/// **No input:**
///
/// ```rust,ignore
/// use metaxy_macro::rpc_mutation;
///
/// #[rpc_mutation]
/// async fn reset_counter() -> u32 {
///     0
/// }
/// ```
///
/// **With a struct input:**
///
/// ```rust,ignore
/// use metaxy_macro::rpc_mutation;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUserInput {
///     name: String,
///     email: String,
/// }
///
/// #[rpc_mutation]
/// async fn create_user(input: CreateUserInput) -> String {
///     format!("Created {}", input.name)
/// }
/// ```
///
/// **Returning `Result`:**
///
/// ```rust,ignore
/// use metaxy_macro::rpc_mutation;
///
/// #[rpc_mutation]
/// async fn delete_user(id: u32) -> Result<String, String> {
///     if id == 0 {
///         Err("cannot delete root user".into())
///     } else {
///         Ok(format!("deleted user {}", id))
///     }
/// }
/// ```
///
/// # Initialization
///
/// Mutations support the `init` and `timeout` attributes (but not `cache`):
///
/// ```rust,ignore
/// #[rpc_mutation(init = "setup")]
/// async fn create_order(input: OrderInput, state: &AppState) -> Order {
///     state.pool.query("...").await
/// }
/// ```
///
/// # Idempotent
///
/// Use the bare `idempotent` flag to mark a mutation as safe to retry.
/// This is **metadata-only** — the server-side handler is unchanged.
/// The generated TypeScript client uses this flag to gate automatic
/// retries for the marked mutations (queries are always retryable).
///
/// ```rust,ignore
/// #[rpc_mutation(idempotent)]
/// async fn upsert_setting(input: Setting) -> Setting { /* ... */ }
///
/// #[rpc_mutation(idempotent, timeout = "30s")]
/// async fn archive_order(id: u64) -> bool { /* ... */ }
/// ```
///
/// **Note:** `idempotent` is rejected on `#[rpc_query]` because queries
/// are inherently idempotent (GET requests are always safe to retry).
///
/// # Compile errors
///
/// The macro rejects functions with more than one parameter:
///
/// ```rust,compile_fail,ignore
/// #[rpc_mutation]
/// async fn bad(a: String, b: u32) -> String { todo!() }
/// // error: RPC handlers accept at most one input parameter
/// ```
#[proc_macro_attribute]
pub fn rpc_mutation(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = match parse_handler_attrs(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };
    if attrs.cache_config.is_some() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "rpc_mutation does not support cache/stale attributes",
        )
        .to_compile_error()
        .into();
    }
    let input_fn = parse_macro_input!(item as ItemFn);
    build_handler(input_fn, HandlerKind::Mutation, attrs)
        .map(Into::into)
        .unwrap_or_else(|e| e.to_compile_error().into())
}

#[cfg(test)]
mod tests;
