#![crate_type = "proc-macro"]
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn pandoc_compile_html(_items: TokenStream) -> TokenStream {
    "".parse().unwrap()
}
