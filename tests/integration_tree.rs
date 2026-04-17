use pandoc_generator::pandoc_compile_html;

// ── Pandoc integration tests using the actual macro ─────────────────────────

// Test with existing test assets (contains index.md and image.svg)
pandoc_compile_html! {
    mod_name = single_mod,
    tree_name = SINGLE_TREE,
    content = [ compile_from_path(path: "tests/assets/content", route: "/") ],
    input_format = Markdown,
    output_format = Html,
    options = [],
    nproc = 1
}

#[test]
fn test_single_file_pipeline() {
    use single_mod::{ContentTree, SINGLE_TREE};

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
    collect_all_html(&*SINGLE_TREE, &mut all);
    assert!(!all.is_empty(), "No HTML content generated");
    let found = all.iter().any(|s| s.contains("image.svg") || s.contains("<img"));
    assert!(found, "generated HTML should reference image.svg or contain an <img> tag");
}

// Test with existing test assets (directory)
pandoc_compile_html! {
    mod_name = dir_mod,
    tree_name = DIR_TREE,
    content = [ compile_from_path(path: "tests/assets/content", route: "/") ],
    input_format = Markdown,
    output_format = Html,
    options = [],
    nproc = 1
}

#[test]
fn test_directory_pipeline() {
    use dir_mod::{ContentTree, DIR_TREE};

    fn count_html_nodes(t: &ContentTree) -> usize {
        match t {
            ContentTree::Html { .. } => 1,
            ContentTree::Nested { elements, .. } => elements.iter().map(count_html_nodes).sum(),
            _ => 0,
        }
    }

    let count = count_html_nodes(&*DIR_TREE);
    assert!(count >= 1, "Should have generated at least one HTML node");
}

// Test with route parameter
pandoc_compile_html! {
    mod_name = route_mod,
    tree_name = ROUTE_TREE,
    content = [ compile_from_path(path: "tests/assets/content", route: "/custom") ],
    input_format = Markdown,
    output_format = Html,
    options = [],
    nproc = 1
}

#[test]
fn test_with_route() {
    use route_mod::{ContentTree, ROUTE_TREE};

    fn check_route(t: &ContentTree, found: &mut bool) {
        match t {
            ContentTree::Html { route, .. } => {
                if route.is_some() {
                    *found = true;
                }
            }
            ContentTree::Nested { route, elements, .. } => {
                if route.is_some() {
                    *found = true;
                }
                for e in elements {
                    check_route(e, found);
                }
            }
            _ => {}
        }
    }

    let mut found = false;
    check_route(&*ROUTE_TREE, &mut found);
    assert!(found, "Route should be set when provided");
}

// ── Tests with mod files and nested structures ───────────────────────────────

// Test with mod files and nested directories
pandoc_compile_html! {
    mod_name = mod_test_mod,
    tree_name = MOD_TEST_TREE,
    content = [ compile_from_path(path: "tests/assets/mod_test", route: "/") ],
    input_format = Markdown,
    output_format = Html,
    options = [],
    nproc = 1
}

#[test]
fn test_mod_file_creates_nested_node() {
    use mod_test_mod::{ContentTree, MOD_TEST_TREE};

    // Verify the tree has nested structure for sections
    fn count_all_nested(t: &ContentTree) -> usize {
        match t {
            ContentTree::Nested { elements, .. } => {
                1 + elements.iter().map(count_all_nested).sum::<usize>()
            }
            ContentTree::Html { .. } | ContentTree::Special { .. } => 0,
        }
    }

    let nested_count = count_all_nested(&*MOD_TEST_TREE);
    assert!(nested_count >= 2, "Should have at least 2 nested nodes (sections)");
}

#[test]
fn test_mod_file_content_in_nested_node() {
    use mod_test_mod::{ContentTree, MOD_TEST_TREE};

    // Collect all nested node names
    fn collect_nested_names(t: &ContentTree, names: &mut Vec<&str>) {
        match t {
            ContentTree::Nested { name, elements, .. } => {
                names.push(name);
                for e in elements {
                    collect_nested_names(e, names);
                }
            }
            ContentTree::Html { .. } | ContentTree::Special { .. } => {}
        }
    }

    let mut names = Vec::new();
    collect_nested_names(&*MOD_TEST_TREE, &mut names);

    // Should have section and subsection as nested nodes
    assert!(names.contains(&"section"), "Should have 'section' as a nested node");
    assert!(names.contains(&"subsection"), "Should have 'subsection' as a nested node");
}

#[test]
fn test_deeply_nested_structure_preserved() {
    use mod_test_mod::{ContentTree, MOD_TEST_TREE};

    // Count depth levels
    fn get_depth(t: &ContentTree, current: usize) -> usize {
        match t {
            ContentTree::Nested { elements, .. } => {
                if elements.is_empty() {
                    current
                } else {
                    elements.iter().map(|e| get_depth(e, current + 1)).max().unwrap_or(current)
                }
            }
            ContentTree::Html { .. } | ContentTree::Special { .. } => current,
        }
    }

    let depth = get_depth(&*MOD_TEST_TREE, 0);
    assert!(depth >= 3, "Should have at least 3 levels of nesting");
}

#[test]
fn test_mod_file_html_content_in_nested() {
    use mod_test_mod::{ContentTree, MOD_TEST_TREE};

    // Collect HTML content from nested nodes (mod files become nested nodes with HTML)
    fn collect_nested_html(t: &ContentTree, htmls: &mut Vec<&str>) {
        match t {
            ContentTree::Nested { elements, .. } => {
                for e in elements {
                    collect_nested_html(e, htmls);
                }
            }
            ContentTree::Html { content, .. } => {
                htmls.push(content);
            }
            ContentTree::Special { .. } => {}
        }
    }

    let mut htmls = Vec::new();
    collect_nested_html(&*MOD_TEST_TREE, &mut htmls);

    // Verify some expected content is present
    let combined = htmls.join(" ");
    assert!(combined.contains("Welcome"), "Should contain 'Welcome' from index.md");
    assert!(combined.contains("Section"), "Should contain 'Section' from mod file");
}

#[test]
fn test_section_without_mod_file_still_nestable() {
    use mod_test_mod::{ContentTree, MOD_TEST_TREE};

    // Count all HTML nodes (files become Html nodes)
    fn count_html_nodes(t: &ContentTree) -> usize {
        match t {
            ContentTree::Html { .. } => 1,
            ContentTree::Nested { elements, .. } => elements.iter().map(count_html_nodes).sum(),
            ContentTree::Special { .. } => 0,
        }
    }

    let count = count_html_nodes(&*MOD_TEST_TREE);
    // Should have: index.md, about.md, page1.md, page2.md, detail.md, info.md = 6 HTML nodes
    assert!(count >= 6, "Should have at least 6 HTML nodes (files)");
}

#[test]
fn test_route_on_nested_nodes() {
    pandoc_compile_html! {
        mod_name = route_nested_mod,
        tree_name = ROUTE_NESTED_TREE,
        content = [ compile_from_path(path: "tests/assets/mod_test", route: "/docs") ],
        input_format = Markdown,
        output_format = Html,
        options = [],
        nproc = 1
    }

    use route_nested_mod::{ContentTree, ROUTE_NESTED_TREE};

    // Check that nested nodes have routes set
    fn check_nested_routes(t: &ContentTree, found: &mut bool) {
        match t {
            ContentTree::Nested { route, elements, .. } => {
                if route.is_some() {
                    *found = true;
                }
                for e in elements {
                    check_nested_routes(e, found);
                }
            }
            ContentTree::Html { .. } | ContentTree::Special { .. } => {}
        }
    }

    let mut found = false;
    check_nested_routes(&*ROUTE_NESTED_TREE, &mut found);
    assert!(found, "Nested nodes should have routes when provided");
}

#[test]
fn test_multiple_mod_files_produce_multiple_nested_nodes() {
    use mod_test_mod::{ContentTree, MOD_TEST_TREE};

    // Count all Nested nodes
    fn count_all_nested(t: &ContentTree) -> usize {
        match t {
            ContentTree::Nested { elements, .. } => {
                1 + elements.iter().map(count_all_nested).sum::<usize>()
            }
            ContentTree::Html { .. } | ContentTree::Special { .. } => 0,
        }
    }

    let nested_count = count_all_nested(&*MOD_TEST_TREE);
    // Should have at least: section, subsection, and potentially more
    assert!(nested_count >= 2, "Should have multiple nested nodes from mod files");
}

#[test]
fn test_regular_files_become_html_nodes() {
    use mod_test_mod::{ContentTree, MOD_TEST_TREE};

    // Collect all HTML node names
    fn collect_html_names(t: &ContentTree, names: &mut Vec<&str>) {
        match t {
            ContentTree::Html { name, .. } => {
                names.push(name);
            }
            ContentTree::Nested { elements, .. } => {
                for e in elements {
                    collect_html_names(e, names);
                }
            }
            ContentTree::Special { .. } => {}
        }
    }

    let mut names = Vec::new();
    collect_html_names(&*MOD_TEST_TREE, &mut names);

    // Should have regular files as HTML nodes
    assert!(names.contains(&"index"), "Should have 'index' as an HTML node");
    assert!(names.contains(&"about"), "Should have 'about' as an HTML node");
    assert!(names.contains(&"page1"), "Should have 'page1' as an HTML node");
    assert!(names.contains(&"page2"), "Should have 'page2' as an HTML node");
    assert!(names.contains(&"detail"), "Should have 'detail' as an HTML node");
    assert!(names.contains(&"info"), "Should have 'info' as an HTML node");
}
