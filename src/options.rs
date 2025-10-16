use std::path::PathBuf;

use pandoc::PandocOption;
use proc_macro2::Span;
use syn::{bracketed, parse::Parse, ExprLit, Ident, Lit, Token};

use crate::arg::parse_pandoc_options;

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(rootdir);
    custom_keyword!(options);
    custom_keyword!(nproc);
    custom_keyword!(name);
}


pub(crate) struct Options {
    pub name: Ident,
    pub rootdir: PathBuf,
    pub pandoc_options: Vec<PandocOption>,
    pub nproc: usize
}

impl Parse for Options {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<keywords::name>()?;
        let _ = input.parse::<Token![=]>()?;
        let name = input.parse::<Ident>()?;

        let _ = input.parse::<keywords::rootdir>()?;
        let _ = input.parse::<Token![=]>()?;
        let rootdir_expr = input.parse::<ExprLit>()?;

        let rootdir = match rootdir_expr.lit {
            Lit::Str(s) => PathBuf::from(s.value()),
            _ => return Err(syn::Error::new(Span::call_site(), "rootdir needs to be a path"))
        };

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

        Ok(Self { name, rootdir, pandoc_options, nproc })
    }
}
