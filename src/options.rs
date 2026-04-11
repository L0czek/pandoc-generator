use pandoc::{InputFormat, MarkdownExtension, OutputFormat, PandocOption};
use proc_macro2::Span;
use syn::{
    ExprLit, Ident, Lit, Token, bracketed, parenthesized, parse::Parse, punctuated::Punctuated,
};

use crate::arg::parse_pandoc_options;

/// Parse markdown extensions from a bracketed list
fn parse_markdown_extensions(input: &syn::parse::ParseStream) -> syn::Result<Vec<MarkdownExtension>> {
    let mut exts = Vec::new();
    if input.peek(syn::token::Bracket) {
        let ext_stream;
        bracketed!(ext_stream in input);
        let ids = Punctuated::<Ident, Token![,]>::parse_terminated(&ext_stream)?;
        for id in ids.iter() {
            let key = id.to_string().to_lowercase();
            let ext = match key.as_str() {
                "smart" => MarkdownExtension::Smart,
                "attributes" => MarkdownExtension::Attributes,
                "escaped_line_breaks" | "escapedlinebreaks" => MarkdownExtension::EscapedLineBreaks,
                "blank_before_header" | "blankbeforeheader" => MarkdownExtension::BlankBeforeHeader,
                "header_attributes" | "headerattributes" => MarkdownExtension::HeaderAttributes,
                "auto_identifiers" | "autoidentifiers" => MarkdownExtension::AutoIdentifiers,
                "implicit_header_references" | "implicitheaderreferences" => MarkdownExtension::ImplicitHeaderReferences,
                "fenced_divs" | "fenceddivs" => MarkdownExtension::FencedDivs,
                "fenced_code_blocks" | "fencedcodeblocks" => MarkdownExtension::FencedCodeBlocks,
                "backtick_code_blocks" | "backtickcodeblocks" => MarkdownExtension::BacktickCodeBlocks,
                "fenced_code_attributes" | "fencedcodeattributes" => MarkdownExtension::FencedCodeAttributes,
                "line_blocks" | "lineblocks" => MarkdownExtension::LineBlocks,
                "fancy_lists" | "fancylists" => MarkdownExtension::FancyLists,
                "startnum" => MarkdownExtension::Startnum,
                "task_lists" | "tasklists" => MarkdownExtension::TaskLists,
                "definition_lists" | "definitionlists" => MarkdownExtension::DefinitionLists,
                "example_lists" | "examplelists" => MarkdownExtension::ExampleLists,
                "table_captions" | "tablecaptions" => MarkdownExtension::TableCaptions,
                "simple_tables" | "simpletables" => MarkdownExtension::SimpleTables,
                "multiline_tables" | "multilinetables" => MarkdownExtension::MultilineTables,
                "grid_tables" | "gridtables" => MarkdownExtension::GridTables,
                "pipe_tables" | "pipetables" => MarkdownExtension::PipeTables,
                "pandoc_title_block" | "pandoctitleblock" => MarkdownExtension::PandocTitleBlock,
                "yaml_metadata_block" | "yamlmetadatablock" => MarkdownExtension::YamlMetadataBlock,
                "all_symbols_escapable" | "allsymbolsescapable" => MarkdownExtension::AllSymbolsEscapable,
                "intraword_underscores" | "intrawordunderscores" => MarkdownExtension::IntrawordUnderscores,
                "strikeout" => MarkdownExtension::Strikeout,
                "superscript" => MarkdownExtension::Superscript,
                "subscript" => MarkdownExtension::Subscript,
                "inline_code_attributes" | "inlinecodeattributes" => MarkdownExtension::InlineCodeAttributes,
                "tex_math_dollars" | "texmathdollars" => MarkdownExtension::TexMathDollars,
                "raw_attribute" | "rawattribute" => MarkdownExtension::RawAttribute,
                "raw_html" | "rawhtml" => MarkdownExtension::RawHtml,
                "markdown_in_html_blocks" | "markdowninhtmlblocks" => MarkdownExtension::MarkdownInHtmlBlocks,
                "native_divs" | "nativedivs" => MarkdownExtension::NativeDivs,
                "native_spans" | "nativespans" => MarkdownExtension::NativeSpans,
                "bracketed_spans" | "bracketedspans" => MarkdownExtension::BracketedSpans,
                "raw_tex" | "rawtex" => MarkdownExtension::RawTex,
                "latex_macros" | "latexmacros" => MarkdownExtension::LatexMacros,
                "shortcut_reference_links" | "shortcutreferencelinks" => MarkdownExtension::ShortcutReferenceLinks,
                "implicit_figures" | "implicitfigures" => MarkdownExtension::ImplicitFigures,
                "footnotes" => MarkdownExtension::Footnotes,
                "inline_notes" | "inlinenotes" => MarkdownExtension::InlineNotes,
                "citations" => MarkdownExtension::Citations,
                "lists_without_preceding_blankline" | "listswithoutprecedingblankline" => MarkdownExtension::ListsWithoutPrecedingBlankline,
                "hard_line_breaks" | "hardlinebreaks" => MarkdownExtension::HardLineBreaks,
                "ignore_line_breaks" | "ignorelinebreaks" => MarkdownExtension::IgnoreLineBreaks,
                "tex_math_single_backslash" | "texmathsinglebackslash" => MarkdownExtension::TexMathSingleBackslash,
                "tex_math_double_backslash" | "texmathdoublebackslash" => MarkdownExtension::TexMathDoubleBackslash,
                "markdown_attribute" | "markdownattribute" => MarkdownExtension::MarkdownAttribute,
                "mmd_title_block" | "mmdtitleblock" => MarkdownExtension::MmdTitleBlock,
                "abbreviations" => MarkdownExtension::Abbreviations,
                "autolink_bare_uris" | "autolinkbareuris" => MarkdownExtension::AutolinkBareUris,
                "ascii_identifiers" | "asciidentifiers" => MarkdownExtension::AsciiIdentifiers,
                "link_attributes" | "linkattributes" => MarkdownExtension::LinkAttributes,
                "mmd_header_identifiers" | "mmdheaderidentifiers" => MarkdownExtension::MmdHeaderIdentifiers,
                "compact_definition_lists" | "compactdefinitionlists" => MarkdownExtension::CompactDefinitionLists,
                "rebase_relative_paths" | "rebaserelativepaths" => MarkdownExtension::RebaseRelativePaths,
                other => MarkdownExtension::Other(other.to_string()),
            };
            exts.push(ext);
        }
    }
    Ok(exts)
}

/// Parse an input format with optional extensions
fn parse_input_format(input: syn::parse::ParseStream) -> syn::Result<(InputFormat, Vec<MarkdownExtension>)> {
    let fmt_ident: Ident = input.parse()?;
    let base = fmt_ident.to_string().to_lowercase();

    let fmt = match base.as_str() {
        "native" => InputFormat::Native,
        "json" => InputFormat::Json,
        "markdown" => InputFormat::Markdown,
        "markdown_strict" => InputFormat::MarkdownStrict,
        "markdown_phpextra" => InputFormat::MarkdownPhpextra,
        "markdown_github" => InputFormat::MarkdownGithub,
        "commonmark" => InputFormat::Commonmark,
        "commonmark_x" | "commonmarkx" => InputFormat::CommonmarkX,
        "textile" => InputFormat::Textile,
        "rst" => InputFormat::Rst,
        "rtf" => InputFormat::Rtf,
        "html" => InputFormat::Html,
        "docbook" => InputFormat::DocBook,
        "t2t" => InputFormat::T2t,
        "docx" => InputFormat::Docx,
        "epub" => InputFormat::Epub,
        "opml" => InputFormat::Opml,
        "org" => InputFormat::Org,
        "mediawiki" => InputFormat::MediaWiki,
        "twiki" => InputFormat::Twiki,
        "haddock" => InputFormat::Haddock,
        "latex" => InputFormat::Latex,
        other => InputFormat::Other(other.to_string()),
    };

    let exts = parse_markdown_extensions(&input)?;
    Ok((fmt, exts))
}

/// Parse an output format with optional extensions
fn parse_output_format(input: syn::parse::ParseStream) -> syn::Result<(OutputFormat, Vec<MarkdownExtension>)> {
    let fmt_ident: Ident = input.parse()?;
    let base = fmt_ident.to_string().to_lowercase();

    let fmt = match base.as_str() {
        "native" => OutputFormat::Native,
        "json" => OutputFormat::Json,
        "plain" => OutputFormat::Plain,
        "markdown" => OutputFormat::Markdown,
        "markdown_strict" => OutputFormat::MarkdownStrict,
        "markdown_phpextra" => OutputFormat::MarkdownPhpextra,
        "markdown_github" => OutputFormat::MarkdownGithub,
        "commonmark" => OutputFormat::Commonmark,
        "commonmark_x" | "commonmarkx" => OutputFormat::CommonmarkX,
        "rst" => OutputFormat::Rst,
        "html" => OutputFormat::Html,
        "html5" => OutputFormat::Html5,
        "latex" => OutputFormat::Latex,
        "beamer" => OutputFormat::Beamer,
        "context" => OutputFormat::Context,
        "pdf" => OutputFormat::Pdf,
        "man" => OutputFormat::Man,
        "mediawiki" => OutputFormat::MediaWiki,
        "dokuwiki" => OutputFormat::Dokuwiki,
        "textile" => OutputFormat::Textile,
        "org" => OutputFormat::Org,
        "texinfo" => OutputFormat::Texinfo,
        "opml" => OutputFormat::Opml,
        "docbook" => OutputFormat::Docbook,
        "open_document" | "opendocument" => OutputFormat::OpenDocument,
        "odt" => OutputFormat::Odt,
        "docx" => OutputFormat::Docx,
        other => OutputFormat::Other(other.to_string()),
    };

    let exts = parse_markdown_extensions(&input)?;
    Ok((fmt, exts))
}

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
    custom_keyword!(path);
    custom_keyword!(ty);
    custom_keyword!(route);
    custom_keyword!(input_format);
    custom_keyword!(output_format);
}

pub(crate) enum Element {
    CompileFromPath { path: String, route: Option<String> },

    Special { ty: String },
}

impl Parse for Element {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        let parse_string_arg = |arg: &syn::parse::ParseBuffer<'_>| {
            let expr = arg.parse::<ExprLit>();

            match expr {
                Ok(ExprLit {
                    lit: Lit::Str(s), ..
                }) => Ok(s.value()),
                Ok(_) => Err(syn::Error::new(
                    Span::call_site(),
                    "Expected a string literal",
                )),
                Err(e) => Err(e),
            }
        };

        if lookahead.peek(keywords::special) {
            let _ = input.parse::<keywords::special>()?;
            let arg;
            parenthesized!(arg in input);

            let _ = arg.parse::<keywords::ty>()?;
            let _ = arg.parse::<Token![:]>()?;
            let ty = parse_string_arg(&arg)?;

            Ok(Element::Special { ty })
        } else if lookahead.peek(keywords::compile_from_path) {
            let _ = input.parse::<keywords::compile_from_path>()?;
            let arg;
            parenthesized!(arg in input);

            let _ = arg.parse::<keywords::path>()?;
            let _ = arg.parse::<Token![:]>()?;
            let path = parse_string_arg(&arg)?;

            let route = if let Ok(_) = arg.parse::<Token![,]>() {
                let _ = arg.parse::<keywords::route>()?;
                let _ = arg.parse::<Token![:]>()?;
                Some(parse_string_arg(&arg)?)
            } else {
                None
            };

            Ok(Element::CompileFromPath { path, route })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) struct Options {
    pub mod_name: Ident,
    pub tree_name: Ident,
    pub content: Punctuated<Element, Token![,]>,
    pub input_format: Option<(InputFormat, Vec<MarkdownExtension>)>,
    pub output_format: Option<(OutputFormat, Vec<MarkdownExtension>)>,
    pub pandoc_options: Vec<PandocOption>,
    pub nproc: usize,
}

impl Parse for Options {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse arguments in any order
        let mut mod_name: Option<Ident> = None;
        let mut tree_name: Option<Ident> = None;
        let mut content: Option<Punctuated<Element, Token![,]>> = None;
        let mut input_format: Option<(InputFormat, Vec<MarkdownExtension>)> = None;
        let mut output_format: Option<(OutputFormat, Vec<MarkdownExtension>)> = None;
        let mut pandoc_options: Option<Vec<PandocOption>> = None;
        let mut nproc: Option<usize> = None;

        // Helper function to parse format with extensions for input format
        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(keywords::mod_name) {
                if mod_name.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "mod_name specified multiple times",
                    ));
                }
                let _ = input.parse::<keywords::mod_name>()?;
                let _ = input.parse::<Token![=]>()?;
                mod_name = Some(input.parse::<Ident>()?);
            } else if lookahead.peek(keywords::tree_name) {
                if tree_name.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "tree_name specified multiple times",
                    ));
                }
                let _ = input.parse::<keywords::tree_name>()?;
                let _ = input.parse::<Token![=]>()?;
                tree_name = Some(input.parse::<Ident>()?);
            } else if lookahead.peek(keywords::content) {
                if content.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "content specified multiple times",
                    ));
                }
                let _ = input.parse::<keywords::content>()?;
                let _ = input.parse::<Token![=]>()?;
                let elements_stream;
                bracketed!(elements_stream in input);
                content = Some(Punctuated::parse_separated_nonempty(&elements_stream)?);
            } else if lookahead.peek(keywords::input_format) {
                if input_format.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "input_format specified multiple times",
                    ));
                }
                let _ = input.parse::<keywords::input_format>()?;
                let _ = input.parse::<Token![=]>()?;
                input_format = Some(parse_input_format(input)?);
            } else if lookahead.peek(keywords::output_format) {
                if output_format.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "output_format specified multiple times",
                    ));
                }
                let _ = input.parse::<keywords::output_format>()?;
                let _ = input.parse::<Token![=]>()?;
                output_format = Some(parse_output_format(input)?);
            } else if lookahead.peek(keywords::options) {
                if pandoc_options.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "options specified multiple times",
                    ));
                }
                let _ = input.parse::<keywords::options>()?;
                let _ = input.parse::<Token![=]>()?;
                let option_stream;
                bracketed!(option_stream in input);
                pandoc_options = Some(parse_pandoc_options(option_stream.parse()?)?);
            } else if lookahead.peek(keywords::nproc) {
                if nproc.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "nproc specified multiple times",
                    ));
                }
                let _ = input.parse::<keywords::nproc>()?;
                let _ = input.parse::<Token![=]>()?;
                let expr = input.parse::<ExprLit>()?;
                match expr.lit {
                    Lit::Int(n) => nproc = Some(n.base10_parse()?),
                    _ => {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "nproc argument should be a number",
                        ));
                    }
                }
            } else {
                return Err(lookahead.error());
            }

            // Parse optional comma separator
            let _ = input.parse::<Token![,]>();
        }

        // Validate required arguments
        let mod_name = mod_name.ok_or_else(|| syn::Error::new(Span::call_site(), "mod_name is required"))?;
        let tree_name = tree_name.ok_or_else(|| syn::Error::new(Span::call_site(), "tree_name is required"))?;
        let content = content.ok_or_else(|| syn::Error::new(Span::call_site(), "content is required"))?;

        Ok(Self {
            mod_name,
            tree_name,
            content,
            input_format,
            output_format,
            pandoc_options: pandoc_options.unwrap_or_default(),
            nproc: nproc.unwrap_or(1),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: convert CamelCase or PascalCase identifier into snake_case expected by pandoc's Display
    fn camel_to_snake(s: &str) -> String {
        let mut out = String::new();
        let mut prev_lower = false;
        for c in s.chars() {
            if c == '_' {
                out.push('_');
                prev_lower = false;
                continue;
            }

            if c.is_ascii_uppercase() {
                if prev_lower {
                    out.push('_');
                }
                out.push(c.to_ascii_lowercase());
                prev_lower = false;
            } else {
                out.push(c);
                prev_lower = true;
            }
        }
        out
    }

    fn build_base_options(fmt_section: &str) -> String {
        format!(
            "mod_name = my_mod, tree_name = MY_TREE, content = [ compile_from_path(path: \"content\", route: \"/\") ], {}, options = [], nproc = 1",
            fmt_section
        )
    }

    #[test]
    fn human_readable_single_case() {
        let src =
            build_base_options("input_format = Markdown[Smart, RawHtml], output_format = Html");
        let opts = syn::parse_str::<Options>(&src).expect("failed to parse options");

        // readable assertions using to_string for comparison
        let (in_fmt, in_exts) = opts.input_format.expect("input_format missing");
        assert_eq!(
            in_fmt.to_string(),
            "markdown",
            "expected input format to be 'markdown'"
        );
        let in_exts_str: Vec<String> = in_exts.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(
            in_exts_str,
            vec!["smart".to_string(), "raw_html".to_string()],
            "expected markdown extensions to match"
        );

        let (out_fmt, out_exts) = opts.output_format.expect("output_format missing");
        assert_eq!(
            out_fmt.to_string(),
            "html",
            "expected output format to be 'html'"
        );
        assert!(out_exts.is_empty(), "expected no output extensions");
    }

    #[test]
    fn all_input_formats_supported_readably() {
        let cases = vec![
            "Native",
            "Json",
            "Markdown",
            "Markdown_Strict",
            "Markdown_PhpExtra",
            "Markdown_Github",
            "Commonmark",
            "Commonmark_X",
            "Textile",
            "Rst",
            "Rtf",
            "Html",
            "T2t",
            "Docx",
            "Epub",
            "Opml",
            "Org",
            "MediaWiki",
            "Twiki",
            "Haddock",
            "Latex",
            "DocBook",
        ];

        for ident in cases.iter() {
            let ident_token = ident.replace('-', "_");
            let section = format!("input_format = {}, output_format = Html", ident_token);
            let src = build_base_options(&section);
            let opts = syn::parse_str::<Options>(&src)
                .unwrap_or_else(|e| panic!("parse failed for {}: {}", ident, e));
            let (fmt, _exts) = opts.input_format.expect("missing input_format");
            let got = fmt.to_string().replace('_', "").to_lowercase();
            let want = camel_to_snake(&ident_token).replace('_', "").to_lowercase();
            assert_eq!(got, want, "input_format mismatch for {}", ident);
        }
    }

    #[test]
    fn all_output_formats_supported_readably() {
        let cases = vec![
            "Native",
            "Json",
            "Plain",
            "Markdown",
            "Markdown_Strict",
            "Markdown_PhpExtra",
            "Markdown_Github",
            "Commonmark",
            "Commonmark_X",
            "Rst",
            "Html",
            "Html5",
            "Latex",
            "Beamer",
            "Context",
            "Pdf",
            "Man",
            "MediaWiki",
            "Dokuwiki",
            "Textile",
            "Org",
            "Texinfo",
            "Opml",
            "Docbook",
            "OpenDocument",
            "Odt",
            "Docx",
            "Haddock",
            "Rtf",
            "Epub",
            "Epub3",
            "Fb2",
            "Asciidoc",
            "Icml",
            "Slidy",
            "Slideous",
            "Dzslides",
            "Revealjs",
            "S5",
        ];

        for ident in cases.iter() {
            let ident_token = ident.replace('-', "_");
            let section = format!("input_format = Markdown, output_format = {}", ident_token);
            let src = build_base_options(&section);
            let opts = syn::parse_str::<Options>(&src)
                .unwrap_or_else(|e| panic!("parse failed for {}: {}", ident, e));
            let (fmt, _exts) = opts.output_format.expect("missing output_format");
            let got = fmt.to_string().replace('_', "").to_lowercase();
            let want = camel_to_snake(&ident_token).replace('_', "").to_lowercase();
            assert_eq!(got, want, "output_format mismatch for {}", ident);
        }
    }

    #[test]
    fn markdown_extensions_all_variants() {
        let exts = vec![
            "Smart",
            "Attributes",
            "EscapedLineBreaks",
            "BlankBeforeHeader",
            "HeaderAttributes",
            "AutoIdentifiers",
            "ImplicitHeaderReferences",
            "FencedDivs",
            "FencedCodeBlocks",
            "BacktickCodeBlocks",
            "FencedCodeAttributes",
            "LineBlocks",
            "FancyLists",
            "Startnum",
            "TaskLists",
            "DefinitionLists",
            "ExampleLists",
            "TableCaptions",
            "SimpleTables",
            "MultilineTables",
            "GridTables",
            "PipeTables",
            "PandocTitleBlock",
            "YamlMetadataBlock",
            "AllSymbolsEscapable",
            "IntrawordUnderscores",
            "Strikeout",
            "Superscript",
            "Subscript",
            "InlineCodeAttributes",
            "TexMathDollars",
            "RawAttribute",
            "RawHtml",
            "MarkdownInHtmlBlocks",
            "NativeDivs",
            "NativeSpans",
            "BracketedSpans",
            "RawTex",
            "LatexMacros",
            "ShortcutReferenceLinks",
            "ImplicitFigures",
            "Footnotes",
            "InlineNotes",
            "Citations",
            "ListsWithoutPrecedingBlankline",
            "HardLineBreaks",
            "IgnoreLineBreaks",
            "TexMathSingleBackslash",
            "TexMathDoubleBackslash",
            "MarkdownAttribute",
            "MmdTitleBlock",
            "Abbreviations",
            "AutolinkBareUris",
            "AsciiIdentifiers",
            "LinkAttributes",
            "MmdHeaderIdentifiers",
            "CompactDefinitionLists",
            "RebaseRelativePaths",
        ];

        // build a single bracketed list: Markdown[Ext1, Ext2, ...]
        let ext_list = exts.join(", ");
        let section = format!(
            "input_format = Markdown[{}], output_format = Html",
            ext_list
        );
        let src = build_base_options(&section);
        let opts =
            syn::parse_str::<Options>(&src).expect("failed to parse options with many extensions");
        let (_fmt, parsed_exts) = opts.input_format.expect("missing input_format");

        let parsed_strs: Vec<String> = parsed_exts.into_iter().map(|e| e.to_string()).collect();
        let expected: Vec<String> = exts.iter().map(|e| camel_to_snake(e)).collect();

        // Normalize by removing underscores for a stable comparison against pandoc's Display
        let parsed_norm: Vec<String> = parsed_strs
            .iter()
            .map(|s| s.replace('_', "").to_lowercase())
            .collect();
        let expected_norm: Vec<String> = expected
            .iter()
            .map(|s| s.replace('_', "").to_lowercase())
            .collect();

        assert_eq!(
            parsed_norm.len(),
            expected_norm.len(),
            "extension count mismatch"
        );
        assert_eq!(
            parsed_norm, expected_norm,
            "parsed extensions did not match expected list"
        );
    }

    #[test]
    fn usage_example_multiline_raw_string() {
        let src = r#"
mod_name = my_mod,
tree_name = MY_TREE,
content = [
    compile_from_path(path: "content", route: "/"),
    special(ty: "custom")
],
input_format = Markdown[Smart, RawHtml],
output_format = Html,
options = [],
nproc = 4
"#;

        let opts = syn::parse_str::<Options>(src).expect("failed to parse usage example");

        assert_eq!(opts.mod_name.to_string(), "my_mod");
        assert_eq!(opts.tree_name.to_string(), "MY_TREE");
        assert_eq!(opts.content.len(), 2);

        let (in_fmt, in_exts) = opts.input_format.expect("input_format missing");
        assert_eq!(in_fmt.to_string().replace('_', ""), "markdown");
        let in_exts_str: Vec<String> = in_exts.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(
            in_exts_str,
            vec!["smart".to_string(), "raw_html".to_string()]
        );

        let (out_fmt, out_exts) = opts.output_format.expect("output_format missing");
        assert_eq!(out_fmt.to_string(), "html");
        assert!(out_exts.is_empty());

        assert_eq!(opts.pandoc_options.len(), 0);
        assert_eq!(opts.nproc, 4);
    }
}
