# pandoc-generator

> A Rust procedural macro that compiles Markdown to HTML and wraps it in a nested data structure because why not.

Built for fun and probably overengineered beyond all reasonable limits.

## Overview

The `pandoc_compile_html!` macro:

- Recursively scans directories for source files
- Compiles Markdown (or other formats) to HTML using Pandoc
- Supports parallel processing with configurable thread count
- Generates a nested `ContentTree` enum for easy traversal
- Supports numeric prefix ordering for deterministic directory sorting

## Basic Usage

```rust
pandoc_compile_html!(
    mod_name = my_mod,
    tree_name = MY_TREE,
    content = [
        compile_from_path(path: "content", route: "/"),
    ],
    input_format = Markdown[Smart, RawHtml],
    output_format = Html,
    options = [],
    nproc = 4
);
```

### Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `mod_name` | Name of the generated Rust module | Yes |
| `tree_name` | Name of the lazy_static ContentTree variable | Yes |
| `content` | List of content sources to process | Yes |
| `input_format` | Source format with optional extensions | No (default: Markdown) |
| `output_format` | Target format with optional extensions | No (default: Html) |
| `options` | Pandoc conversion options | No (default: empty) |
| `nproc` | Number of parallel processing threads | No (default: 1) |
| `source_ext` | File extension to filter (e.g., "md") | No (default: all files) |

## Content Sources

### compile_from_path

Processes a directory or single file:

```rust
compile_from_path(path: "path/to/content", route: "/")
```

- `path`: Directory or file path to process
- `route`: Optional route string attached to all generated nodes

### special

Inserts a special node type:

```rust
special(ty: "header")
```

## Format Specifications

### Input Format

Base formats supported: Native, Json, Markdown, Markdown_Strict, Markdown_PhpExtra, Markdown_Github, Commonmark, Commonmark_X, Textile, Rst, Rtf, Html, DocBook, T2t, Docx, Epub, Opml, Org, MediaWiki, Twiki, Haddock, Latex

Markdown extensions (optional bracketed list):

```rust
input_format = Markdown[Smart, RawHtml, Footnotes, Citations]
```

### Output Format

Base formats supported: Native, Json, Plain, Markdown, Html, Html5, Latex, Beamer, Pdf, and more

```rust
output_format = Html
output_format = Pdf[PdfEngine { "xelatex" }]
```

## Pandoc Options

Supported options include:

```rust
options = [
    Smart,
    TableOfContents,
    Standalone,
    NoWrap,
    DataDir { "data" },
    Filter { "filter.py" },
    Meta { "title", "My Document" },
    TabStop { 4 }
]
```

## Directory Structure and Sorting

### Numeric Prefix Ordering

Directories with names like `01-section`, `02-subsection` are sorted by their numeric prefix (1, 2, etc.) rather than alphabetically. This allows authors to control output order.

Example directory structure:

```
content/
├── 01-introduction/
├── 02-basics/
├── 03-advanced/
└── z-appendices/
```

Sorting order: `01-introduction`, `02-basics`, `03-advanced`, `z-appendices`

### Mod Files

A special `__mod__.<ext>` file in a directory is treated as a leaf node instead of recursing into the directory. This is useful for section intro pages.

Example:

```
section/
├── __mod__.md    # Becomes a single HTML node
├── page1.md      # These are ignored when __mod__.md exists
└── page2.md
```

## Generated ContentTree

The macro generates a module with:

```rust
pub enum ContentTree {
    Special { ty: &'static str },
    Html { name: &'static str, content: &'static str, route: Option<&'static str> },
    Nested { name: &'static str, elements: Vec<ContentTree>, route: Option<&'static str> },
}

pub static ref TREE_NAME: ContentTree;
```

## Example: Complete Site Generation

```rust
pandoc_compile_html!(
    mod_name = site,
    tree_name = SITE_TREE,
    content = [
        compile_from_path(path: "docs", route: "/docs"),
        compile_from_path(path: "blog", route: "/blog"),
    ],
    input_format = Markdown[Smart, Footnotes, Citations],
    output_format = Html5,
    options = [
        Standalone,
        TableOfContents,
        TableOfContentsDepth { 3 },
        DataDir { "pandoc" }
    ],
    nproc = 8
);

// Use the generated tree
use site::{ContentTree, SITE_TREE};

fn render_tree(tree: &ContentTree) {
    match tree {
        ContentTree::Html { name, content, .. } => {
            println!("{}:\n{}", name, content);
        }
        ContentTree::Nested { name, elements, .. } => {
            println!("=== {} ===", name);
            for elem in elements {
                render_tree(elem);
            }
        }
        ContentTree::Special { ty } => {
            println!("Special node: {}", ty);
        }
    }
}

render_tree(&*SITE_TREE);
```

## Testing

Run the integration tests:

```bash
cargo test
```

The test suite includes tests for:

- Single file compilation
- Directory traversal
- Route parameter handling
- Mod file processing
- Nested structure preservation
- Numeric prefix sorting
- Multiple content sources

## Building

```bash
cargo build
cargo check
```
