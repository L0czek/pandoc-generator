use std::{fs, io, path::PathBuf};

pub(crate) enum TreeElement {
    File(PathBuf),
    Nested(PathBuf, Vec<TreeElement>)
}

pub(crate) struct FsTree {
    pub tree: TreeElement,
    pub route: Option<String>
}

impl FsTree {
    pub(crate) fn construct(rootdir: PathBuf, route: &Option<String>) -> Result<Self, io::Error> {
        if rootdir.is_file() {
            Ok(Self {
                tree: TreeElement::File(rootdir),
                route: route.clone()
            })
        } else {
            let components= FsTree::make_tree(&rootdir)?;

            Ok(Self {
                tree: TreeElement::Nested(rootdir, components),
                route: route.clone()
            })
        }
    }

    fn make_tree(dir: &PathBuf) -> Result<Vec<TreeElement>, io::Error> {
        let mut components = Vec::new();

        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                components.push(TreeElement::Nested(path.clone(), FsTree::make_tree(&path)?));
            } else {
                components.push(TreeElement::File(path));
            }
        }

        components.sort_by_key(|i| {
            let path = match i {
                TreeElement::File(path) => path,
                TreeElement::Nested(path, _) => path
            };

            let index = path.iter().last().unwrap()
                .to_str().unwrap();

            index.to_owned()
        });


        Ok(components)
    }

    fn list_files(components: &[TreeElement], list: &mut Vec<PathBuf>) {
        for el in components.iter() {
            match el {
                TreeElement::File(path) => list.push(path.clone()),
                TreeElement::Nested(_,subtree) =>
                    Self::list_files(subtree, list),
            }
        }
    }

    pub(crate) fn get_all_src_files(&self) -> Vec<PathBuf> {
        let mut list = Vec::new();

        match &self.tree {
            TreeElement::File(path) => list.push(path.clone()),
            TreeElement::Nested(_, components) => Self::list_files(components, &mut list),
        }

        list
    }
}
