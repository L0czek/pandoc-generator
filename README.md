# pandoc-generator

Example usage of the `pandoc_compile_html!` macro with format + extension list using a bracketed `Punctuated` list:

```rust
pandoc_compile_html!(
    mod_name = my_mod,
    tree_name = MY_TREE,
    content = [
        compile_from_path(path: "content", route: "/"),
    ],
    input_format = Markdown[Smart, RawHtml],
    output_format = Html
);
```

- `input_format = Markdown[Smart, RawHtml]` shows the new syntax: an identifier for the base format followed by an optional bracketed, comma-separated list of markdown extensions.
- `output_format = Html` is the simple case with no extensions.

Run checks:

```bash
cargo check
cargo test --no-run
```
