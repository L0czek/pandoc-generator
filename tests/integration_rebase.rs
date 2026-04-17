use pandoc_generator::pandoc_compile_html;

pandoc_compile_html! {
    mod_name = my_mod,
    tree_name = MY_TREE,
    content = [ compile_from_path(path: "tests/assets/content", route: "/") ],
    input_format = Markdown[RebaseRelativePaths],
    output_format = Html,
    options = [],
    nproc = 1
}

#[test]
fn test_rebase_relative_paths() {
    use my_mod::{ContentTree, MY_TREE};

    fn collect_all_html<'a>(t: &'a ContentTree, out: &mut Vec<&'a str>) {
        match t {
            ContentTree::Html { content, .. } => out.push(content),
            ContentTree::Nested { elements, .. } => {
                for e in elements {
                    collect_all_html(e, out);
                }
            }
            _ => {}
        }
    }

    let mut all = Vec::new();
    collect_all_html(&*MY_TREE, &mut all);
    assert!(!all.is_empty(), "No HTML content generated");
    let found = all.iter().any(|s| s.contains("image.svg") || s.contains("<img"));
    assert!(found, "generated HTML should reference image.svg or contain an <img> tag");
}
