use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};

struct HtmlWalker<'a, W: Write> {
    found_article: bool,
    title: String,
    ignore_elements: HashSet<String>,
    handle: &'a Handle,
    writer: &'a mut BufWriter<W>,
}
#[derive(Clone, Copy, PartialEq)]
enum State {
    Search,
    Title,
    P,
    Pre,
    Ol,
    Ul,
    Table,
}

impl<'a, W: std::io::Write> HtmlWalker<'a, W> {
    fn new(handle: &'a Handle, writer: &'a mut BufWriter<W>) -> Self {
        let mut ignore_elements = HashSet::new();
        let title = String::new();
        ignore_elements.insert("link".to_string());
        ignore_elements.insert("meta".to_string());
        ignore_elements.insert("script".to_string());
        ignore_elements.insert("style".to_string());
        HtmlWalker {
            found_article: false,
            title,
            ignore_elements,
            handle,
            writer,
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
            NodeData::Text { ref contents } => match e {
                State::Search => {
                    if self.found_article {
                        let text = contents.borrow().replace('\n', " ");
                        let text = text.trim();
                        if !text.is_empty() {
                            writeln!(self.writer, "{}", text).expect("buffer overflow");
                        }
                    }
                }
                State::Title => {
                    self.title = contents.borrow().clone().into();
                    //no longer needed to look into head elements
                    self.ignore_elements.insert("head".to_string());
                }
                State::Pre => {
                    write!(self.writer, "{}", contents.borrow()).expect("buffer overflow")
                }
                _ => write!(self.writer, "{}", contents.borrow().replace('\n', " "))
                    .expect("buffer overflow"),
            },
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let name = std::str::from_utf8(name.local.as_bytes()).unwrap();
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
        if self.ignore_elements.contains(name) {
            return;
        }
        if is_article(name, attrs) {
            self.found_article = true;
            self.walk_children(node, state);
        } else if self.found_article {
            match name {
                "p" => self.print_element(node, State::P),
                "br" => writeln!(self.writer).expect("buffer overflow"),
                "pre" => {
                    self.print_pre(node);
                    writeln!(self.writer).expect("buffer overflow");
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
            match name {
                "title" => self.walk_children(node, State::Title),
                _ => self.walk_children(node, state),
            }
        }
    }
    fn print_text(&mut self, handle: &Handle, e: State) {
        for child in handle.children.borrow().iter() {
            match child.data {
                NodeData::Text { ref contents } => {
                    write!(self.writer, "{}", contents.borrow().replace('\n', " "))
                        .expect("buffer overflow");
                }

                NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } => {
                    let name = std::str::from_utf8(name.local.as_bytes()).unwrap();
                    self.handle_element(name, attrs, child, e);
                }
                _ => {}
            }
        }
    }

    fn print_element(&mut self, handle: &Handle, e: State) {
        self.print_text(handle, e);
        writeln!(self.writer).expect("buffer overflow");
    }
    fn print_pre(&mut self, handle: &Handle) {
        for child in handle.children.borrow().iter() {
            match child.data {
                NodeData::Text { ref contents } => {
                    write!(self.writer, "{}", contents.borrow()).expect("buffer overflow");
                }

                NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } => {
                    let string = std::str::from_utf8(name.local.as_bytes()).unwrap();
                    if string == "br" {
                        writeln!(self.writer).expect("buffer overflow");
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
                    writeln!(self.writer).expect("buffer overflow");
                } else if string == "th" || string == "td" {
                    write!(self.writer, "\t").expect("buffer overflow");
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
                    write!(self.writer, "• ").expect("buffer overflow");
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
                    write!(self.writer, "{}. ", pos).expect("buffer overflow");
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
            let mut writer =
                BufWriter::new(File::create("/tmp/article.txt").expect("error creating file"));
            let mut walker = HtmlWalker::new(&dom.document, &mut writer);
            walker.walk();
            if !walker.found_article {
                walker.found_article = true;
                walker.walk();
            }
            if !walker.title.is_empty() {
                let filename = String::from("/tmp/") + &walker.title + ".txt";
                fs::rename("/tmp/article.txt", &filename)
                    .unwrap_or_else(|_| panic!("error renaming article.txt to {}", filename));
            }
            Ok(())
        }
        None => Ok(()),
    }
}
