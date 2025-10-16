use proc_macro2::Span;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Token,
};
use syn::token::Brace;
use std::{fmt::Display, path::PathBuf};

// Replace this with the actual path to your crate defining PandocOption.
use pandoc::{PandocOption, TrackChanges};

/// Represents one parsed option (either `Name` or `Name { args... }`)
enum PandocOptionExpr {
    Unit(Ident),
    Variant(Ident, Vec<Expr>),
}

/// List of Pandoc options (top-level parse result)
struct PandocOptionList(Vec<PandocOptionExpr>);

impl Parse for PandocOptionList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = Vec::new();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            if input.peek(Brace) {
                let content;
                braced!(content in input);
                let args = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?
                    .into_iter()
                    .collect::<Vec<_>>();
                out.push(PandocOptionExpr::Variant(ident, args));
            } else {
                out.push(PandocOptionExpr::Unit(ident));
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(Self(out))
    }
}

/// Extract string literal from expression
fn parse_string_arg(args: &[Expr], idx: usize) -> syn::Result<String> {
    if let Some(Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(litstr),
        ..
    })) = args.get(idx)
    {
        Ok(litstr.value())
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            format!("Expected string literal at index {}", idx),
        ))
    }
}

/// Extract integer literal from expression
fn parse_number_arg<T: std::str::FromStr>(args: &[Expr], idx: usize) -> syn::Result<T>
    where <T as std::str::FromStr>::Err: Display
{
    if let Some(Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(litint),
        ..
    })) = args.get(idx)
    {
        litint.base10_parse::<T>().map_err(|_| {
            syn::Error::new_spanned(litint, "Failed to parse integer literal into target type")
        })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            format!("Expected integer literal at index {}", idx),
        ))
    }
}

/// Parse the token stream into `Vec<PandocOption>`
pub fn parse_pandoc_options(tokens: proc_macro2::TokenStream) -> syn::Result<Vec<PandocOption>> {
    let parsed = syn::parse2::<PandocOptionList>(tokens)?;
    let mut out = Vec::new();

    for item in parsed.0 {
        match item {
            PandocOptionExpr::Unit(ident) => {
                let name = ident.to_string();
                let val = match name.as_str() {
                    "Strict" => PandocOption::Strict,
                    "ParseRaw" => PandocOption::ParseRaw,
                    "Smart" => PandocOption::Smart,
                    "OldDashes" => PandocOption::OldDashes,
                    "Normalize" => PandocOption::Normalize,
                    "PreserveTabs" => PandocOption::PreserveTabs,
                    "Standalone" => PandocOption::Standalone,
                    "NoWrap" => PandocOption::NoWrap,
                    "TableOfContents" => PandocOption::TableOfContents,
                    "NoHighlight" => PandocOption::NoHighlight,
                    "SelfContained" => PandocOption::SelfContained,
                    "Offline" => PandocOption::Offline,
                    "Html5" => PandocOption::Html5,
                    "HtmlQTags" => PandocOption::HtmlQTags,
                    "Ascii" => PandocOption::Ascii,
                    "ReferenceLinks" => PandocOption::ReferenceLinks,
                    "AtxHeaders" => PandocOption::AtxHeaders,
                    "NumberSections" => PandocOption::NumberSections,
                    "NoTexLigatures" => PandocOption::NoTexLigatures,
                    "Listings" => PandocOption::Listings,
                    "Incremental" => PandocOption::Incremental,
                    "SectionDivs" => PandocOption::SectionDivs,
                    "Citeproc" => PandocOption::Citeproc,
                    "Natbib" => PandocOption::Natbib,
                    "Biblatex" => PandocOption::Biblatex,
                    "GladTex" => PandocOption::GladTex,
                    "Trace" => PandocOption::Trace,
                    "DumpArgs" => PandocOption::DumpArgs,
                    "IgnoreArgs" => PandocOption::IgnoreArgs,
                    "Verbose" => PandocOption::Verbose,
                    "Sandbox" => PandocOption::Sandbox,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            format!("Unsupported or unknown unit variant `{}`", name),
                        ))
                    }
                };
                out.push(val);
            }

            PandocOptionExpr::Variant(ident, args) => {
                let name = ident.to_string();

                match name.as_str() {
                    // PathBuf/String options
                    "DataDir" => out.push(PandocOption::DataDir(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "Defaults" => out.push(PandocOption::Defaults(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "BaseHeaderLevel" => out.push(PandocOption::BaseHeaderLevel(parse_number_arg::<u32>(&args, 0)?)),
                    "ShiftHeadingLevelBy" => out.push(PandocOption::ShiftHeadingLevelBy(parse_number_arg::<i32>(&args, 0)?)),
                    "IndentedCodeClasses" => out.push(PandocOption::IndentedCodeClasses(parse_string_arg(&args, 0)?)),
                    "Filter" => out.push(PandocOption::Filter(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "LuaFilter" => out.push(PandocOption::LuaFilter(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "TabStop" => out.push(PandocOption::TabStop(parse_number_arg::<u32>(&args, 0)?)),
                    "ExtractMedia" => out.push(PandocOption::ExtractMedia(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "Template" => out.push(PandocOption::Template(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "Meta" => {
                        let key = parse_string_arg(&args, 0)?;
                        let val = if args.len() > 1 { Some(parse_string_arg(&args, 1)?) } else { None };
                        out.push(PandocOption::Meta(key, val));
                    }
                    "Var" => {
                        let key = parse_string_arg(&args, 0)?;
                        let val = if args.len() > 1 { Some(parse_string_arg(&args, 1)?) } else { None };
                        out.push(PandocOption::Var(key, val));
                    }
                    "PrintDefaultTemplate" => out.push(PandocOption::PrintDefaultTemplate(parse_string_arg(&args, 0)?)),
                    "PrintDefaultDataFile" => out.push(PandocOption::PrintDefaultDataFile(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "Columns" => out.push(PandocOption::Columns(parse_number_arg::<u32>(&args, 0)?)),
                    "TableOfContentsDepth" => out.push(PandocOption::TableOfContentsDepth(parse_number_arg::<u32>(&args, 0)?)),
                    "HighlightStyle" => out.push(PandocOption::HighlightStyle(parse_string_arg(&args, 0)?)),
                    "IncludeInHeader" => out.push(PandocOption::IncludeInHeader(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "IncludeBeforeBody" => out.push(PandocOption::IncludeBeforeBody(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "IncludeAfterBody" => out.push(PandocOption::IncludeAfterBody(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "SlideLevel" => out.push(PandocOption::SlideLevel(parse_number_arg::<u32>(&args, 0)?)),
                    "DefaultImageExtension" => out.push(PandocOption::DefaultImageExtension(parse_string_arg(&args, 0)?)),
                    "IdPrefix" => out.push(PandocOption::IdPrefix(parse_string_arg(&args, 0)?)),
                    "TitlePrefix" => out.push(PandocOption::TitlePrefix(parse_string_arg(&args, 0)?)),
                    "PdfEngine" => out.push(PandocOption::PdfEngine(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "PdfEngineOpt" => out.push(PandocOption::PdfEngineOpt(parse_string_arg(&args, 0)?)),
                    "Bibliography" => out.push(PandocOption::Bibliography(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "Csl" => out.push(PandocOption::Csl(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "CitationAbbreviations" => out.push(PandocOption::CitationAbbreviations(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "ReferenceDoc" => out.push(PandocOption::ReferenceDoc(PathBuf::from(parse_string_arg(&args, 0)?))),
                    "EpubChapterLevel" => out.push(PandocOption::EpubChapterLevel(parse_number_arg::<u32>(&args, 0)?)),
                    "EOL" => out.push(PandocOption::EOL(parse_string_arg(&args, 0)?)),

                    // Enum-type args (simple fallback)
                    "TrackChanges" => {
                        if args.is_empty() {
                            return Err(syn::Error::new_spanned(&ident, "TrackChanges requires one of Accept|Reject|All"));
                        }
                        if let Expr::Path(p) = &args[0] {
                            if p.path.is_ident("Accept") {
                                out.push(PandocOption::TrackChanges(TrackChanges::Accept));
                            } else if p.path.is_ident("Reject") {
                                out.push(PandocOption::TrackChanges(TrackChanges::Reject));
                            } else if p.path.is_ident("All") {
                                out.push(PandocOption::TrackChanges(TrackChanges::All));
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &args[0],
                                    "Invalid TrackChanges variant (expected Accept|Reject|All)",
                                ));
                            }
                        }
                    }

                    // Unsupported / complex
                    _ => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            format!("Variant `{}` with complex/custom type not yet supported", name),
                        ));
                    }
                }
            }
        }
    }

    Ok(out)
}

/// Convert `Vec<PandocOption>` to token stream (for expansion)
fn tokens_from_pandoc_option(opt: &PandocOption) -> proc_macro2::TokenStream {
    match opt {
        PandocOption::Strict => quote! { PandocOption::Strict },
        PandocOption::ParseRaw => quote! { PandocOption::ParseRaw },
        PandocOption::Smart => quote! { PandocOption::Smart },
        PandocOption::Standalone => quote! { PandocOption::Standalone },
        _ => quote! { compile_error!("Unsupported PandocOption for tokenization"); },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_parse_unit_variants() {
        let input = quote! { Strict, ParseRaw, Smart };
        let opts = parse_pandoc_options(input).unwrap();
        assert_eq!(
            opts,
            vec![
                PandocOption::Strict,
                PandocOption::ParseRaw,
                PandocOption::Smart
            ]
        );
    }

    #[test]
    fn test_parse_variant_with_string_arg() {
        let input = quote! { DataDir { "data" } };
        let opts = parse_pandoc_options(input).unwrap();
        match &opts[0] {
            PandocOption::DataDir(p) => assert_eq!(p, &PathBuf::from("data")),
            _ => panic!("Expected DataDir variant"),
        }
    }

    #[test]
    fn test_parse_variant_with_number_arg() {
        let input = quote! { TabStop { 8 } };
        let opts = parse_pandoc_options(input).unwrap();
        match &opts[0] {
            PandocOption::TabStop(n) => assert_eq!(*n, 8),
            _ => panic!("Expected TabStop variant"),
        }
    }

    #[test]
    fn test_parse_track_changes_accept() {
        let input = quote! { TrackChanges { Accept } };
        let opts = parse_pandoc_options(input).unwrap();
        match &opts[0] {
            PandocOption::TrackChanges(TrackChanges::Accept) => {}
            _ => panic!("Expected TrackChanges(Accept)"),
        }
    }

    #[test]
    fn test_unknown_unit_variant_fails() {
        let input = quote! { Foo };
        let err = parse_pandoc_options(input).unwrap_err();
        assert!(err.to_string().contains("Unsupported or unknown unit variant"));
    }

    #[test]
    fn test_unsupported_variant_fails() {
        let input = quote! { SomeUnsupported { "arg" } };
        let err = parse_pandoc_options(input).unwrap_err();
        assert!(err.to_string().contains("not yet supported"));
    }
}
