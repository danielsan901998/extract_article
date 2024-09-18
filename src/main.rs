extern crate markup5ever_rcdom as rcdom;

use html5ever::{parse_document,ParseOpts,tendril::TendrilSink};
use rcdom::{Handle, NodeData, RcDom};
use std::cell::RefCell;
use std::collections::HashSet;

struct HtmlWalker<'a> {
    found_article: bool,
    handle: &'a Handle,
    ignore_elements: HashSet<String>,
}
#[derive(Clone, Copy)]
enum State {
    Search,
    P,
    Pre,
    Ol,
    Ul,
    Table,
}

impl<'a> HtmlWalker<'a> {
    fn new(handle: &'a Handle) -> Self {
        let mut ignore_elements = HashSet::new();
        ignore_elements.insert("head".to_string());
        ignore_elements.insert("script".to_string());
        ignore_elements.insert("style".to_string());
        HtmlWalker {
            found_article: false,
            handle,
            ignore_elements,
        }
    }

    fn walk(&mut self) {
        self.visit(self.handle, State::Search);
    }

    fn walk_children(&mut self, node: &Handle, e: State) {
        for child in node.children.borrow().iter() {
            self.visit(child, e);
        }
    }

    fn visit(&mut self, node: &Handle, e: State) {
        match node.data {
            NodeData::Document => self.walk_children(node, e),
            NodeData::Text { ref contents } => {
                match e {
                    State::Search =>{
                        if self.found_article {
                            let text = contents.borrow().replace('\n', " ");
                            let text = text.trim();
                            if !text.is_empty() {
                                println!("{}", text);
                            }
                        }
                    }
                    State::Pre => {
                        print!("{}", contents.borrow());
                    }
                    _ =>{
                        print!("{}", contents.borrow().replace('\n', " "));
                    }
                }
            }
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let name = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if self.ignore_elements.contains(name) {
                    return;
                }

                self.handle_element(name, attrs, node, e);
            }
            _ => {}
        }
    }
    fn handle_element(
        &mut self,
        name: &str,
        attrs: &RefCell<Vec<html5ever::Attribute>>,
        node: &Handle,
        state: State,
    ) {
        if is_article(name, attrs) {
            self.found_article = true;
            self.walk_children(node, state);
        } else if self.found_article {
            match name {
                "p" => self.print_element(node, State::P),
                "pre" => {
                    self.print_pre(node);
                    println!();
                }
                "ol" => {
                    let mut reversed = false;
                    let mut start = 1;
                    for attr in attrs.borrow().iter() {
                        let name = std::str::from_utf8(attr.name.local.as_bytes()).unwrap();
                        if name == "reversed" {
                            reversed = true;
                        } else if name == "start" {
                            let value = std::str::from_utf8(attr.value.as_bytes()).unwrap();
                            start = value.parse::<i32>().unwrap();
                        }
                    }
                    self.print_ol(node, start, reversed);
                }
                "ul" => self.print_ul(node),
                "table" => self.print_table(node),
                _ if name.starts_with('h') && name.len() == 2 => self.print_element(node, State::P),
                _ => self.walk_children(node, state),
            }
        } else {
            self.walk_children(node, state);
        }
    }
    fn print_text(&mut self, handle: &Handle, e: State) {
        for child in handle.children.borrow().iter() {
            match child.data {
                NodeData::Text { ref contents } => {
                    print!("{}", contents.borrow().replace('\n', " "));
                }

                NodeData::Element { ref name,
                ref attrs,
                .. } => {
                    let name = std::str::from_utf8(name.local.as_bytes()).unwrap();
                    if self.ignore_elements.contains(name) {
                        return;
                    }
                    self.handle_element(name, attrs, child, e);
                }
                _ => {}
            }
        }
    }

    fn print_element(&mut self, handle: &Handle, e: State) {
        self.print_text(handle, e);
        println!();
    }
    fn print_pre(&mut self, handle: &Handle) {
        for child in handle.children.borrow().iter() {
            match child.data {
                NodeData::Text { ref contents } => {
                    print!("{}", contents.borrow());
                }

                NodeData::Element { ref name,
                ref attrs,
                .. } => {
                    let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                    if string == "br" {
                        println!();
                    } else {
                        self.handle_element(string, attrs, child, State::Pre);
                    }
                }
                _ => {}
            }
        }
    }
    fn print_table(&mut self, handle: &Handle) {
        for child in handle.children.borrow().iter() {
            if let NodeData::Element { ref name, .. } = child.data {
                let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if string == "tr" {
                    self.print_table(child);
                    println!();
                } else if string == "th" || string == "td" {
                    print!("\t");
                    self.print_text(child, State::Table);
                } else {
                    self.print_table(child);
                }
            }
        }
    }
    fn print_ul(&mut self, handle: &Handle) {
        for child in handle.children.borrow().iter() {
            if let NodeData::Element { ref name, .. } = child.data {
                let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if string == "li" {
                    print!("• ");
                    self.print_element(child, State::Ul);
                }
            }
        }
    }
    fn print_ol(&mut self, handle: &Handle, start: i32, reversed: bool) {
        let mut pos = start;
        for child in handle.children.borrow().iter() {
            if let NodeData::Element { ref name, .. } = child.data {
                let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                if string == "li" {
                    print!("{}. ", pos);
                    self.print_element(child, State::Ol);
                    if reversed {
                        pos -= 1;
                    } else {
                        pos += 1;
                    }
                }
            }
        }
    }
}

fn is_article(name: &str, attrs: &RefCell<Vec<html5ever::Attribute>>) -> bool {
    if name == "article" {
        return true;
    }
    if name == "div" {
        for attr in attrs.borrow().iter() {
            let name = std::str::from_utf8(attr.name.local.as_bytes()).unwrap();
            if name == "class" {
                let value = std::str::from_utf8(attr.value.as_bytes()).unwrap();
                if value == "post hentry" || value == "post-content" {
                    return true;
                }
            }
        }
    }
    false
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1) {
        Some(url) => {
            let resp = reqwest::get(url).await.unwrap();
            let data = resp.text().await.unwrap();
            let mut opts: ParseOpts = Default::default();
            opts.tree_builder.scripting_enabled = false;
            let dom = parse_document(RcDom::default(), opts)
                .from_utf8()
                .read_from(&mut data.as_bytes())
                .unwrap();
            let mut walker = HtmlWalker::new(&dom.document);
            walker.walk();
            if !walker.found_article {
                walker.found_article = true;
                walker.walk();
            }
            Ok(())
        }
        None => Ok(()),
    }
}
