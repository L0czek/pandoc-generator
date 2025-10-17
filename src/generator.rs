use std::{collections::HashMap, path::PathBuf};

use pandoc::PandocOutput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;
use crate::{tree::TreeElement, FsTree};

pub(crate) fn generate_content_tree(name: Ident, tree: &FsTree, outputs: &HashMap<&&PathBuf, PandocOutput>) -> TokenStream {
    let content = process_tree_element(tree, outputs);

    quote! {
        mod #name {
            #[derive(Debug, Copy, Clone)]
            pub(crate) enum ContentTree {
                Html {
                    name: &'static str,
                    content: &'static str
                },

                Nested {
                    name: &'static str,
                    elements: std::vec::Vec<ContentTree>
                }
            }

            static TREE: ContentTree = ContentTree::Nested {
                name: "ROOT",
                components: #content
            };
        }
    }
}

fn get_name(path: &PathBuf) -> String {
    path.file_name().unwrap().to_os_string().into_string().unwrap()
}

fn process_tree_element(tree: &FsTree, outputs: &HashMap<&&PathBuf, PandocOutput>) -> TokenStream {
    let components = tree.components.iter().map(|i| {
        match i {
            TreeElement::File(path) => {
                let name = get_name(path);
                let content = match outputs.get(&path).unwrap() {
                    PandocOutput::ToBuffer(output) => output,
                    _ => panic!("Pandoc didn't output to pipe?")
                };

                quote! {
                    ContentTree::Html {
                        name: #name,
                        content: #content
                    }
                }
            },

            TreeElement::Nested(path, subtree) => {
                let name = get_name(path);
                let content = process_tree_element(subtree, outputs);

                quote! {
                    ContentTree::Nested {
                        name: #name,
                        elements: #content
                    }
                }
            }
        }
    }).collect::<Vec<TokenStream>>();

    quote! {
        vec![
            #(#components),*
        ]
    }
}
