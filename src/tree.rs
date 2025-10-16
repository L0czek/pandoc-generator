use std::{fs, io, path::PathBuf};

pub(crate) enum TreeElement {
    File(PathBuf),
    Nested(PathBuf, FsTree)
}

pub(crate) struct FsTree {
    pub components: Vec<TreeElement>
}

impl FsTree {
    pub(crate) fn construct(rootdir: &PathBuf) -> Result<Self, io::Error> {
        FsTree::make_tree(rootdir)
    }

    fn make_tree(dir: &PathBuf) -> Result<FsTree, io::Error> {
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


        Ok(FsTree { components })
    }

    fn list_files(&self, list: &mut Vec<PathBuf>) {
        for el in self.components.iter() {
            match el {
                TreeElement::File(path) => list.push(path.clone()),
                TreeElement::Nested(_, tree) => tree.list_files(list),
            }
        }
    }

    pub(crate) fn get_all_src_files(&self) -> Vec<PathBuf> {
        let mut list = Vec::new();
        self.list_files(&mut list);
        list
    }
}
