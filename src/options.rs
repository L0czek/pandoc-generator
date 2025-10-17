use std::path::PathBuf;

use pandoc::PandocOption;
use proc_macro2::Span;
use syn::{braced, bracketed, parenthesized, parse::Parse, punctuated::{self, Punctuated}, token::Token, Expr, ExprLit, Ident, Lit, LitStr, Token};

use crate::arg::parse_pandoc_options;

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(rootdir);
    custom_keyword!(options);
    custom_keyword!(nproc);
    custom_keyword!(mod_name);
    custom_keyword!(tree_name);
    custom_keyword!(content);
    custom_keyword!(compile_from_path);
    custom_keyword!(special);
}

pub(crate) enum Element {
    CompileFromPath(String),
    Special(String)
}

impl Parse for Element {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        let parse_string_arg = |input: &syn::parse::ParseBuffer<'_>| {
            let arg;
            parenthesized!(arg in input);
            let expr = arg.parse::<ExprLit>();

            match expr {
                Ok(ExprLit {
                    lit: Lit::Str(s),
                    ..
                }) => Ok(s.value()),
                Ok(_) => Err(
                    syn::Error::new(
                        Span::call_site(),
                        "Expected a string literal"
                    )
                ),
                Err(e) => Err(e)
            }
        };

        if lookahead.peek(keywords::special) {
            let _ = input.parse::<keywords::special>()?;

            Ok(Element::Special(parse_string_arg(input)?))
        } else if lookahead.peek(keywords::compile_from_path) {
            let _ = input.parse::<keywords::compile_from_path>()?;

            Ok(Element::CompileFromPath(parse_string_arg(input)?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) struct Options {
    pub mod_name: Ident,
    pub tree_name: Ident,
    pub content: Punctuated<Element, Token![,]>,
    pub pandoc_options: Vec<PandocOption>,
    pub nproc: usize
}

impl Parse for Options {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<keywords::mod_name>()?;
        let _ = input.parse::<Token![=]>()?;
        let mod_name = input.parse::<Ident>()?;
        let _ = input.parse::<Token![,]>()?;

        let _ = input.parse::<keywords::tree_name>()?;
        let _ = input.parse::<Token![=]>()?;
        let tree_name = input.parse::<Ident>()?;
        let _ = input.parse::<Token![,]>()?;

        let _ = input.parse::<keywords::content>()?;
        let _ = input.parse::<Token![=]>()?;
        let elements_stream;
        bracketed!(elements_stream in input);
        let content = Punctuated::parse_separated_nonempty(&elements_stream)?;

        let pandoc_options: Vec<PandocOption> = if input.peek(Token![,]) {
            let _ = input.parse::<Token![,]>()?;

            let _ = input.parse::<keywords::options>()?;
            let _ = input.parse::<Token![=]>()?;

            let option_stream;
            bracketed!(option_stream in input);

            parse_pandoc_options(option_stream.parse()?)?
        } else {
            vec![]
        };

        let nproc = if input.peek(Token![,]) {
            let _ = input.parse::<Token![,]>()?;

            let _ = input.parse::<keywords::nproc>()?;
            let _ = input.parse::<Token![=]>()?;

            let expr = input.parse::<ExprLit>()?;

            match expr.lit {
                Lit::Int(n) => n.base10_parse()?,
                _ => return Err(syn::Error::new(Span::call_site(), "nproc argument should be a number"))
            }
        } else {
            1
        };

        Ok(Self { mod_name, tree_name, content, pandoc_options, nproc })
    }
}
