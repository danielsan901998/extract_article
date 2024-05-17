extern crate markup5ever_rcdom as rcdom;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use rcdom::{Handle, NodeData, RcDom};

fn print_text(handle: &Handle) {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Text { ref contents } => {
                print!("{}", contents.borrow().replace('\n'," "))
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

fn print_element(handle: &Handle) {
    print_text(handle);
    println!("");
}
fn print_pre(handle: &Handle) {
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
                    print_pre(child);
                }
            },
                _ => {},
        }
    }
}
fn print_table(handle: &Handle) {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element {
                ref name,
                ..
            } => {
                let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if string == "tr" {
                    print_table(child);
                    println!("");
                }else if string == "th" || string == "td" {
                    print!("\t");
                    print_text(child);
                }else {
                    print_table(child);
                }
            },
                _ => {},
        }
    }
}
fn print_ul(handle: &Handle) {
    for child in handle.children.borrow().iter() {
        match child.data {
            NodeData::Element {
                ref name,
                ..
            } => {
                let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if string =="li" {
                    print!("• ");
                    print_element(child);
                }
            }
            _ => {},

        }
    }
}
fn print_ol(handle: &Handle, start: i32, reversed: bool) {
    let mut pos = start;
    for child in handle.children.borrow().iter() {
        match child.data {
            NodeData::Element {
                ref name,
                ..
            } => {
                let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if string =="li" {
                    print!("{}. ",pos);
                    print_element(child);
                    if reversed {
                        pos=pos-1;
                    }
                    else{
                        pos=pos-1;
                    }
                }
            }
            _ => {},

        }
    }
}

fn walk_children(handle: &Handle, article: bool) -> bool {
    let mut article = article;
    for child in handle.children.borrow().iter() {
        if walk(child,article){
            article=true;
        }
    }
    article
}

fn walk(handle: &Handle, article: bool) -> bool {
    let node = handle;
    let mut article = article;
    match node.data {
        NodeData::Document => {
            walk_children(node,article)
        },
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
            if string =="article" {
                article = true;
            }
            else if article {
                if string == "p" {
                    print_element(handle);
                    return article;
                }
                else if string == "pre" {
                    print_pre(handle);
                    println!("");
                    return article;
                }
                else if string == "ol" {
                    let mut reversed = false;
                    let mut start = 1;
                    for attr in attrs.borrow().iter() {
                        let name = std::str::from_utf8(attr.name.local.as_bytes()).unwrap();
                        if name == "reversed" {
                            reversed=true;
                        }else if name == "start" {
                            let value = std::str::from_utf8(attr.value.as_bytes()).unwrap();
                            start = value.parse::<i32>().unwrap();
                        }
                    }
                    print_ol(handle, start, reversed);
                    return article;
                }
                else if string == "ul" {
                    print_ul(handle);
                    return article;
                }
                else if string == "table" {
                    print_table(handle);
                    return article;
                }
            }
            walk_children(node,article)
        },
            _ => {article},
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    match std::env::args().nth(1) {
        Some(url) =>{
            let resp = reqwest::get(url).await.unwrap();
            let data = resp.text().await.unwrap();
            let dom = parse_document(RcDom::default(), Default::default())
                .from_utf8()
                .read_from(&mut data.as_bytes()).unwrap();
            let article_found=walk(&dom.document, false);
            if !article_found {
                walk(&dom.document, true);
            }
            if !dom.errors.is_empty() {
                eprintln!("\nParse errors:");
                for err in dom.errors.iter() {
                    eprintln!("    {}", err);
                }
            }
            Ok(())
        },
        None =>{
            Ok(())
        }
    }

}
