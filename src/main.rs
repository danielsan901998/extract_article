extern crate markup5ever_rcdom as rcdom;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use rcdom::{Handle, NodeData, RcDom};

struct HtmlWalker<'a> {
    found_article: bool,
    handle: &'a Handle,
}

impl<'a> HtmlWalker<'a> {
    fn new(handle: &'a Handle) -> Self {
        HtmlWalker { found_article: false, handle }
    }

    fn walk(&mut self){
        self.walk_inner(self.handle);
    }

    fn walk_children(&mut self, node : &Handle){
        for child in node.children.borrow().iter() {
            self.walk_inner(child);
        }
    }

    fn walk_inner(&mut self, node : &Handle) {
        match node.data {
            NodeData::Document => {
                self.walk_children(node)
            },
            NodeData::Text { ref contents } => {
                if self.found_article {
                    let text = contents.borrow().replace('\n'," ");
                    let text = text.trim();
                    if !text.is_empty() {
                        println!("{}",text);
                    }
                }
            },
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let name = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if name == "head" || name == "script" {
                    return;
                }
                else if name =="article" {
                    self.found_article = true;
                }
                else if self.found_article {
                    if name == "p" {
                        print_element(node);
                        return;
                    }
                    else if name == "pre" {
                        print_pre(node);
                        println!("");
                        return;
                    }
                    else if name == "ol" {
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
                        print_ol(node, start, reversed);
                        return;
                    }
                    else if name == "ul" {
                        print_ul(node);
                        return;
                    }
                    else if name == "table" {
                        print_table(node);
                        return;
                    }
                    else if name.starts_with('h') && name.len()==2 {
                        print_element(node);
                        return;
                    }
                }
                self.walk_children(node)
            },
                _ => {},
        }
    }
}

fn print_text(handle: &Handle) {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Text { ref contents } => {
                print!("{}", contents.borrow().replace('\n'," "));
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
                print!("{}", contents.borrow());
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
                        pos=pos+1;
                    }
                }
            }
            _ => {},

        }
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
            let mut walker = HtmlWalker::new(&dom.document);
            walker.walk();
            if !walker.found_article {
                walker.found_article=true;
                walker.walk();
            }
            Ok(())
        },
        None =>{
            Ok(())
        }
    }

}
