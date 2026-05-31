use std::{fs, io, path::PathBuf};

#[derive(Debug)]
pub(crate) enum TreeElement {
    File(PathBuf),
    ModFile(PathBuf),
    Nested(PathBuf, Vec<TreeElement>),
}

#[derive(Debug)]
pub(crate) struct FsTree {
    pub tree: TreeElement,
    pub route: Option<String>,
}

impl FsTree {
    pub(crate) fn construct(
        rootdir: PathBuf,
        route: &Option<String>,
        mod_file_name: &str,
        source_ext: &Option<String>,
    ) -> Result<Self, io::Error> {
        if rootdir.is_file() {
            Ok(Self {
                tree: TreeElement::File(rootdir),
                route: route.clone(),
            })
        } else {
            if rootdir.join(mod_file_name).is_file() {
                return Ok(Self {
                    tree: TreeElement::ModFile(rootdir.join(mod_file_name)),
                    route: route.clone()
                });
            }

            let components = FsTree::make_tree(&rootdir, mod_file_name, source_ext)?;

            if components.is_empty() {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "Directory {:?} contains no source files{}",
                        rootdir,
                        source_ext
                            .as_ref()
                            .map(|e| format!(" with extension .{}", e))
                            .unwrap_or_default()
                    ),
                ))
            } else {
                Ok(Self {
                    tree: TreeElement::Nested(rootdir, components),
                    route: route.clone(),
                })
            }
        }
    }

    fn parse_order(path_str: &str) -> u32 {
        // Extract the first component (directory/file name)
        let name = path_str;

        // Find the end of the number part (leading digits)
        let num_str = name
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>();

        if !num_str.is_empty() {
            // Check if followed by '-'
            if name.chars().nth(num_str.len()) == Some('-') {
                // Valid "<number>-" prefix found
                return num_str.parse::<u32>().unwrap_or(0);
            }
        }

        // No numeric prefix found - use max value to sort after numbered items
        u32::MAX
    }

    fn make_tree(dir: &PathBuf, mod_file_name: &str, source_ext: &Option<String>) -> Result<Vec<TreeElement>, io::Error> {
        let mut components = Vec::new();

        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                // Check for __mod__.<ext> file inside this directory
                let mod_path = path.join(mod_file_name);
                if mod_path.is_file() {
                    components.push(TreeElement::ModFile(mod_path));
                    continue;
                }

                let subtree = FsTree::make_tree(&path, mod_file_name, source_ext)?;
                if !subtree.is_empty() {
                    components.push(TreeElement::Nested(path, subtree));
                }
            } else {
                // Only include files matching the source extension (if specified)
                if let Some(ext) = source_ext {
                    if path.extension().and_then(|e| e.to_str()) == Some(ext.as_str()) {
                        components.push(TreeElement::File(path));
                    }
                } else {
                    components.push(TreeElement::File(path));
                }
            }
        }

        components.sort_by_key(|i| {
            let path = match i {
                TreeElement::File(path) => path,
                TreeElement::ModFile(path) => path,
                TreeElement::Nested(path, _) => path,
            };

            let index = path.iter().last().unwrap().to_str().unwrap();
            let order = Self::parse_order(&index);

            (order, index.to_owned())
        });

        Ok(components)
    }

    fn list_files(components: &[TreeElement], list: &mut Vec<PathBuf>) {
        for el in components.iter() {
            match el {
                TreeElement::File(path) | TreeElement::ModFile(path) => {
                    list.push(path.clone());
                }
                TreeElement::Nested(_, subtree) => Self::list_files(subtree, list),
            }
        }
    }

    pub(crate) fn get_all_src_files(&self) -> Vec<PathBuf> {
        let mut list = Vec::new();

        match &self.tree {
            TreeElement::File(path) | TreeElement::ModFile(path) => {
                list.push(path.clone())
            }
            TreeElement::Nested(_, components) => Self::list_files(components, &mut list),
        }

        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Helper: create a temp dir with a unique name, cleaned up on drop.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "{}_{}",
                prefix,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).expect("failed to create temp dir");
            Self { path }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_file(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).expect("failed to write file");
    }

    fn create_dir(parent: &Path, name: &str) -> PathBuf {
        let p = parent.join(name);
        fs::create_dir_all(&p).expect("failed to create dir");
        p
    }

    // ── Basic construction ──────────────────────────────────────────

    #[test]
    fn construct_single_file() {
        let tmp = TempDir::new("tree_single_file");
        let file_path = tmp.path().join("hello.md");
        write_file(tmp.path(), "hello.md", "content");

        let tree = FsTree::construct(file_path.clone(), &None, "__mod__.md", &None).unwrap();
        assert!(matches!(tree.tree, TreeElement::File(ref p) if p == &file_path));
        assert!(tree.route.is_none());
    }

    #[test]
    fn construct_directory_with_files() {
        let tmp = TempDir::new("tree_dir_files");
        write_file(tmp.path(), "a.md", "a");
        write_file(tmp.path(), "b.md", "b");

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => assert_eq!(components.len(), 2),
            other => panic!("expected Nested, got variant at {:?}", name_of(other)),
        }
    }

    #[test]
    fn construct_with_route() {
        let tmp = TempDir::new("tree_route");
        write_file(tmp.path(), "x.md", "x");

        let route = Some("/myroute".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &route, "__mod__.md", &None).unwrap();
        assert_eq!(tree.route, route);
    }

    // ── Empty directory pruning ─────────────────────────────────────

    #[test]
    fn empty_root_dir_returns_error() {
        let tmp = TempDir::new("tree_empty_root");
        // No files at all

        let result = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn empty_subdir_is_pruned() {
        let tmp = TempDir::new("tree_prune_empty");
        write_file(tmp.path(), "root.md", "r");
        let _empty_sub = create_dir(tmp.path(), "empty_sub");
        // empty_sub has no files

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                // Only the root.md file, no Nested for empty_sub
                assert_eq!(components.len(), 1);
                assert!(matches!(&components[0], TreeElement::File(p) if p.file_name().unwrap() == "root.md"));
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn nested_empty_dirs_are_pruned() {
        let tmp = TempDir::new("tree_nested_empty");
        write_file(tmp.path(), "top.md", "t");
        let sub1 = create_dir(tmp.path(), "sub1");
        let _sub2 = create_dir(&sub1, "sub2");
        // sub1/sub2 are both empty

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                assert_eq!(components.len(), 1); // only top.md
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    // ── source_ext filtering ────────────────────────────────────────

    #[test]
    fn source_ext_filters_files() {
        let tmp = TempDir::new("tree_ext_filter");
        write_file(tmp.path(), "a.md", "a");
        write_file(tmp.path(), "b.txt", "b");
        write_file(tmp.path(), "c.md", "c");

        let ext = Some("md".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                assert_eq!(components.len(), 2);
                for c in components {
                    match c {
                        TreeElement::File(p) => {
                            assert_eq!(p.extension().unwrap(), "md");
                        }
                        other => panic!("expected File, got {:?}", name_of(other)),
                    }
                }
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn source_ext_no_match_returns_error() {
        let tmp = TempDir::new("tree_ext_nomatch");
        write_file(tmp.path(), "a.txt", "a");
        write_file(tmp.path(), "b.rs", "b");

        let ext = Some("md".to_string());
        let result = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext);
        assert!(result.is_err());
    }

    #[test]
    fn source_ext_filters_in_subdirs() {
        let tmp = TempDir::new("tree_ext_subdir");
        let sub = create_dir(tmp.path(), "sub");
        write_file(&sub, "good.md", "g");
        write_file(&sub, "bad.txt", "b");

        let ext = Some("md".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                assert_eq!(components.len(), 1); // only sub directory
                match &components[0] {
                    TreeElement::Nested(_, sub_components) => {
                        assert_eq!(sub_components.len(), 1); // only good.md
                    }
                    other => panic!("expected Nested subdir, got {:?}", name_of(other)),
                }
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn source_ext_empty_subdir_with_only_non_matching_is_pruned() {
        let tmp = TempDir::new("tree_ext_prune_sub");
        write_file(tmp.path(), "root.md", "r");
        let sub = create_dir(tmp.path(), "sub");
        write_file(&sub, "ignore.txt", "i");

        let ext = Some("md".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                assert_eq!(components.len(), 1); // only root.md, sub is pruned
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    // ── __mod__.<ext> handling ──────────────────────────────────────

    #[test]
    fn mod_file_creates_leaf_from_directory() {
        let tmp = TempDir::new("tree_mod_leaf");
        let sub = create_dir(tmp.path(), "01-mysection");
        write_file(&sub, "__mod__.md", "mod content");

        let ext = Some("md".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                assert_eq!(components.len(), 1);
                match &components[0] {
                    TreeElement::ModFile(p) => {
                        assert_eq!(p.file_name().unwrap(), "__mod__.md");
                        assert_eq!(p.parent().unwrap().file_name().unwrap(), "01-mysection");
                    }
                    other => panic!("expected ModFile, got {:?}", name_of(other)),
                }
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn mod_file_directory_is_not_nested() {
        let tmp = TempDir::new("tree_mod_not_nested");
        let sub = create_dir(tmp.path(), "mymod");
        write_file(&sub, "__mod__.md", "mod");
        write_file(&sub, "extra.md", "extra");

        let ext = Some("md".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                // ModFile takes priority; directory is NOT recursed
                assert_eq!(components.len(), 1);
                assert!(matches!(&components[0], TreeElement::ModFile(_)));
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn mod_file_standalone_in_current_dir_is_not_skipped() {
        let tmp = TempDir::new("tree_mod_skip");
        write_file(tmp.path(), "__mod__.md", "mod");
        write_file(tmp.path(), "normal.md", "n");

        let ext = Some("md".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext).unwrap();
        match &tree.tree {
            TreeElement::ModFile(p) => assert_eq!(p, &tmp.path().join("__mod__.md")),
            x => panic!("Expected ModFile got: {:?}", x)
        }
    }

    #[test]
    fn mod_file_is_recognized_with_any_source_ext() {
        let tmp = TempDir::new("tree_mod_any_ext");
        let sub = create_dir(tmp.path(), "mymod");
        write_file(&sub, "__mod__.md", "mod");

        // Even with source_ext = None, __mod__.md should be recognized as ModFile
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                // Only the ModFile entry, directory is not recursed
                assert_eq!(components.len(), 1);
                assert!(matches!(&components[0], TreeElement::ModFile(p) if p.file_name().unwrap() == "__mod__.md"));
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    // ── get_all_src_files ───────────────────────────────────────────

    #[test]
    fn get_all_src_files_from_file() {
        let tmp = TempDir::new("tree_src_file");
        let file_path = tmp.path().join("single.md");
        write_file(tmp.path(), "single.md", "s");

        let tree = FsTree::construct(file_path.clone(), &None, "__mod__.md", &None).unwrap();
        let srcs = tree.get_all_src_files();
        assert_eq!(srcs, vec![file_path]);
    }

    #[test]
    fn get_all_src_files_includes_mod_file() {
        let tmp = TempDir::new("tree_src_mod");
        let sub = create_dir(tmp.path(), "mymod");
        write_file(&sub, "__mod__.md", "mod");
        write_file(tmp.path(), "root.md", "r");

        let ext = Some("md".to_string());
        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &ext).unwrap();
        let srcs = tree.get_all_src_files();
        assert_eq!(srcs.len(), 2);

        let names: Vec<String> = srcs.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert!(names.contains(&"__mod__.md".to_string()));
        assert!(names.contains(&"root.md".to_string()));
    }

    #[test]
    fn get_all_src_files_recursive() {
        let tmp = TempDir::new("tree_src_recursive");
        write_file(tmp.path(), "top.md", "t");
        let sub = create_dir(tmp.path(), "sub");
        write_file(&sub, "inner.md", "i");
        let deep = create_dir(&sub, "deep");
        write_file(&deep, "bottom.md", "b");

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        let srcs = tree.get_all_src_files();
        assert_eq!(srcs.len(), 3);
    }

    // ── Sorting ─────────────────────────────────────────────────────

    #[test]
    fn components_are_sorted_alphabetically() {
        let tmp = TempDir::new("tree_sort");
        write_file(tmp.path(), "z.md", "z");
        write_file(tmp.path(), "a.md", "a");
        write_file(tmp.path(), "m.md", "m");

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                let names: Vec<&str> = components.iter().map(|c| match c {
                    TreeElement::File(p) => p.file_name().unwrap().to_str().unwrap(),
                    other => panic!("expected File, got {:?}", name_of(other)),
                }).collect();
                assert_eq!(names, vec!["a.md", "m.md", "z.md"]);
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn components_are_sorted_by_numeric_prefix() {
        let tmp = TempDir::new("tree_numeric_sort");
        write_file(tmp.path(), "z.md", "z");
        let sub1 = create_dir(tmp.path(), "02-second");
        write_file(&sub1, "content.md", "c2");
        let sub3 = create_dir(tmp.path(), "01-first");
        write_file(&sub3, "content.md", "c1");
        let sub4 = create_dir(tmp.path(), "03-third");
        write_file(&sub4, "content.md", "c3");

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                let names: Vec<&str> = components.iter().map(|c| match c {
                    TreeElement::Nested(p, _) => p.file_name().unwrap().to_str().unwrap(),
                    TreeElement::File(p) => p.file_name().unwrap().to_str().unwrap(),
                    other => panic!("expected Nested/File, got {:?}", name_of(other)),
                }).collect();
                // Should be ordered by numeric prefix: 01-first, 02-second, 03-third, z.md
                assert_eq!(names, vec!["01-first", "02-second", "03-third", "z.md"]);
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn components_with_numeric_prefix_mixed_with_non_numeric() {
        let tmp = TempDir::new("tree_mixed_sort");
        let sub_no_num = create_dir(tmp.path(), "no_prefix");
        write_file(&sub_no_num, "a.md", "a");
        let sub_02 = create_dir(tmp.path(), "02-with-num");
        write_file(&sub_02, "b.md", "b");
        let sub_01 = create_dir(tmp.path(), "01-with-num");
        write_file(&sub_01, "c.md", "c");

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                let names: Vec<&str> = components.iter().map(|c| match c {
                    TreeElement::Nested(p, _) => p.file_name().unwrap().to_str().unwrap(),
                    other => panic!("expected Nested, got {:?}", name_of(other)),
                }).collect();
                // Numeric prefixes come first, then non-prefixed alphabetically
                assert_eq!(names, vec!["01-with-num", "02-with-num", "no_prefix"]);
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn non_sequential_numeric_prefixes_sort_correctly() {
        let tmp = TempDir::new("tree_nonseq_sort");
        // Create directories with gaps in numbering: 2-a, 3-b, 1-c, and non-prefixed
        let sub_2a = create_dir(tmp.path(), "2-a");
        write_file(&sub_2a, "x.md", "a");
        let sub_3b = create_dir(tmp.path(), "3-b");
        write_file(&sub_3b, "x.md", "b");
        let sub_1c = create_dir(tmp.path(), "1-c");
        write_file(&sub_1c, "x.md", "c");
        let sub_no_num = create_dir(tmp.path(), "unordered");
        write_file(&sub_no_num, "x.md", "u");

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                let names: Vec<&str> = components.iter().map(|c| match c {
                    TreeElement::Nested(p, _) => p.file_name().unwrap().to_str().unwrap(),
                    TreeElement::File(p) => p.file_name().unwrap().to_str().unwrap(),
                    other => panic!("expected Nested/File, got {:?}", name_of(other)),
                }).collect();
                // Should sort by numeric prefix regardless of gaps: 1-c, 2-a, 3-b, unordered
                assert_eq!(names, vec!["1-c", "2-a", "3-b", "unordered"]);
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    #[test]
    fn numeric_prefix_edge_cases() {
        let tmp = TempDir::new("tree_edge_cases");
        // Test edge cases:
        // - 001-leading-zeros (should parse as 1)
        // - 5-no-trailing-dash (DOES match - has dash after 5, sorts as 5)
        // - -no-leading-num (should NOT match - no number)
        // - 10-larger-number (should parse as 10)
        let sub_001 = create_dir(tmp.path(), "001-leading-zeros");
        write_file(&sub_001, "x.md", "a");
        let sub_no_dash = create_dir(tmp.path(), "5-no-trailing-dash");
        write_file(&sub_no_dash, "x.md", "b");
        let sub_no_num = create_dir(tmp.path(), "-no-leading-num");
        write_file(&sub_no_num, "x.md", "c");
        let sub_10 = create_dir(tmp.path(), "10-larger-number");
        write_file(&sub_10, "x.md", "d");

        let tree = FsTree::construct(tmp.path().clone(), &None, "__mod__.md", &None).unwrap();
        match &tree.tree {
            TreeElement::Nested(_, components) => {
                let names: Vec<&str> = components.iter().map(|c| match c {
                    TreeElement::Nested(p, _) => p.file_name().unwrap().to_str().unwrap(),
                    other => panic!("expected Nested, got {:?}", name_of(other)),
                }).collect();
                // Sorted by numeric prefix: 001-leading-zeros (1), 5-no-trailing-dash (5), 10-larger-number (10), -no-leading-num (u32::MAX)
                assert_eq!(names, vec!["001-leading-zeros", "5-no-trailing-dash", "10-larger-number", "-no-leading-num"]);
            }
            other => panic!("expected Nested, got {:?}", name_of(other)),
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────

    fn name_of(elem: &TreeElement) -> &'static str {
        match elem {
            TreeElement::File(_) => "File",
            TreeElement::ModFile(_) => "ModFile",
            TreeElement::Nested(_, _) => "Nested",
        }
    }
}
