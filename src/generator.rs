use std::{collections::HashMap, path::PathBuf};

use pandoc::PandocOutput;
use proc_macro2::TokenStream;
use quote::quote;
use crate::{tree::TreeElement, Element, FsTree, Options};

pub(crate) fn generate_content_tree(options: &Options, trees: &[FsTree], outputs: &HashMap<&&PathBuf, PandocOutput>) -> TokenStream {
    let mod_name = &options.mod_name;
    let tree_name = &options.tree_name;
    let subtrees= trees.iter().map(|i| process_tree_element(&i.tree, outputs)).collect::<Vec<TokenStream>>();

    let mut component = Vec::new();
    let mut subtree_it = subtrees.into_iter();
    for element in options.content.iter() {
        match element {
            Element::Special(s) => component.push(quote! {
                ContentTree::Special { ty: #s }
            }),

            Element::CompileFromPath(_) => {
                let code = subtree_it.next().unwrap();

                component.push(code);
            },
        }
    }

    let content = quote! {
        vec![
            #(#component),*
        ]
    };

    quote! {
        pub(crate) mod #mod_name {
            use lazy_static::lazy_static;

            #[derive(Debug, PartialEq, Eq)]
            pub(crate) enum ContentTree {
                Special {
                    ty: &'static str
                },

                Html {
                    name: &'static str,
                    content: &'static str
                },

                Nested {
                    name: &'static str,
                    elements: std::vec::Vec<ContentTree>
                }
            }


            lazy_static! {
                pub(crate) static ref #tree_name: ContentTree = ContentTree::Nested {
                    name: "ROOT",
                    elements: #content
                };
            }
        }
    }
}

fn get_name(path: &PathBuf) -> String {
    path.file_name().unwrap().to_os_string().into_string().unwrap()
}

fn process_tree_element(tree: &TreeElement, outputs: &HashMap<&&PathBuf, PandocOutput>) -> TokenStream {
    match tree {
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
            let content = process_subtree_elements(subtree, outputs);

            quote! {
                ContentTree::Nested {
                    name: #name,
                    elements: #content
                }
            }
        }
    }
}

fn process_subtree_elements(tree: &[TreeElement], outputs: &HashMap<&&PathBuf, PandocOutput>) -> TokenStream {
    let components = tree.iter().map(|i| {
        process_tree_element(i, outputs)
    }).collect::<Vec<TokenStream>>();

    quote! {
        vec![
            #(#components),*
        ]
    }
}
