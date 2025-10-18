use std::{collections::HashMap, path::PathBuf};

use pandoc::PandocOutput;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use crate::{tree::TreeElement, Element, FsTree, Options};

fn generate_option<T: ToTokens>(arg: &Option<T>) -> TokenStream {
    match arg {
        Some(v) => quote! { Some(#v) },
        None => quote! { None }
    }
}

pub(crate) fn generate_content_tree(options: &Options, trees: &[FsTree], outputs: &HashMap<&&PathBuf, PandocOutput>) -> TokenStream {
    let mod_name = &options.mod_name;
    let tree_name = &options.tree_name;
    let subtrees= trees.iter().map(|i| process_tree_element(&i.tree, outputs, &i.route)).collect::<Vec<TokenStream>>();

    let mut component = Vec::new();
    let mut subtree_it = subtrees.into_iter();
    for element in options.content.iter() {
        match element {
            Element::Special {
                ty
            } => component.push(quote! {
                ContentTree::Special { ty: #ty }
            }),

            Element::CompileFromPath { .. } => {
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
                    content: &'static str,
                    route: Option<&'static str>
                },

                Nested {
                    name: &'static str,
                    elements: std::vec::Vec<ContentTree>,
                    route: Option<&'static str>
                }
            }


            lazy_static! {
                pub(crate) static ref #tree_name: ContentTree = ContentTree::Nested {
                    name: "ROOT",
                    elements: #content,
                    route: None
                };
            }
        }
    }
}

fn get_name(path: &PathBuf) -> String {
    path.file_name().unwrap().to_os_string().into_string().unwrap()
}

fn process_tree_element(tree: &TreeElement, outputs: &HashMap<&&PathBuf, PandocOutput>, route: &Option<String>) -> TokenStream {
    let route = generate_option(route);
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
                    content: #content,
                    route: #route
                }
            }
        },

        TreeElement::Nested(path, subtree) => {
            let name = get_name(path);
            let content = process_subtree_elements(subtree, outputs);

            quote! {
                ContentTree::Nested {
                    name: #name,
                    elements: #content,
                    route: #route
                }
            }
        }
    }
}

fn process_subtree_elements(tree: &[TreeElement], outputs: &HashMap<&&PathBuf, PandocOutput>) -> TokenStream {
    let components = tree.iter().map(|i| {
        process_tree_element(i, outputs, &None)
    }).collect::<Vec<TokenStream>>();

    quote! {
        vec![
            #(#components),*
        ]
    }
}
