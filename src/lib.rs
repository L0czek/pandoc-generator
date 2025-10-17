#![crate_type = "proc-macro"]
extern crate proc_macro;

use std::{collections::HashMap, path::PathBuf, sync::mpsc::channel};

use generator::generate_content_tree;
use options::{Element, Options};
use pandoc::Pandoc;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use threadpool::ThreadPool;
use tqdm::tqdm;
use tree::FsTree;

mod arg;
mod generator;
mod options;
mod tree;

#[proc_macro]
pub fn pandoc_compile_html(items: TokenStream) -> TokenStream {
    let options: Options = parse_macro_input!(items);

    let pool = ThreadPool::new(options.nproc);

    let mut trees = Vec::new();
    let mut srcs = Vec::new();

    for element in options.content.iter() {
        if let Element::CompileFromPath(path) = element {
            let tree = FsTree::construct(PathBuf::from(path))
                    .expect(format!("Failed to explore dir {}", path).as_str());
            srcs.extend(tree.get_all_src_files().into_iter());
            trees.push(tree);
        }
    }

    let mut out = HashMap::new();

    println!("Starting pandoc");
    for src in tqdm(srcs.iter()) {
        let (tx, rx) = channel();
        let src_file = src.clone();
        let pandoc_options = options.pandoc_options.clone();

        pool.execute(move || {
            let mut pandoc = Pandoc::new();
            pandoc.add_options(&pandoc_options);
            pandoc.set_input(pandoc::InputKind::Files(vec![src_file]));
            pandoc.set_output(pandoc::OutputKind::Pipe);
            tx.send(pandoc.execute()).unwrap();
        });

        out.insert(src, rx);
    }

    let mut outputs = HashMap::new();
    println!("Gathering results");
    for (path, rx) in tqdm(out.iter_mut()) {
        let output= rx.recv()
            .expect("Failed to read result from pandoc")
            .expect(format!("Pandoc failed to convert the file {:?}", path).as_str());

        outputs.insert(path, output);
    }

    let out = generate_content_tree(&options, &trees, &outputs).into();

    out
}

