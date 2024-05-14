extern crate markup5ever_rcdom as rcdom;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use rcdom::{Handle, NodeData, RcDom};

fn print_text(handle: &Handle) {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Text { ref contents } => {
                print!("{}", contents.borrow())
            },

            NodeData::Element {
                ref name,
                ..
            } => {
                let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if string =="br" {
                    println!("");
                }else{
                    print_text(child);
                }
            },
                _ => {},
        }
    }
}
fn print_p(handle: &Handle) {
    print_text(handle);
    println!("");
}

fn walk(handle: &Handle, article: bool) {
    let node = handle;
    let mut article = article;
    match node.data {
        NodeData::Element {
            ref name,
            ..
        } => {
            let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
            if string =="article" {
                article = true;
            }
            else if article {
                if string =="p" {
                    print_p(handle);
                    return;
                }
                else if string =="pre" {
                    print_p(handle);
                    return;
                }
            }
        },
            _ => {},
    }
    for child in node.children.borrow().iter() {
        walk(child,article);
    }

}

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .from_file(path)
        .unwrap();
    walk(&dom.document, false);

    if !dom.errors.is_empty() {
        println!("\nParse errors:");
        for err in dom.errors.iter() {
            println!("    {}", err);
        }
    }
}
