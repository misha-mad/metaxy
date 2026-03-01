use crate::attrs::HandlerAttrs;
use syn::{ItemFn, Type};

pub(super) fn parse_fn(code: &str) -> ItemFn {
    syn::parse_str(code).unwrap()
}

pub(super) fn parse_type(code: &str) -> Type {
    syn::parse_str(code).unwrap()
}

pub(super) fn no_attrs() -> HandlerAttrs {
    HandlerAttrs::default()
}
