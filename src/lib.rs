use markup5ever_rcdom::{Handle, NodeData};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Write;

#[derive(Debug, PartialEq)]
pub enum SimpleSelector {
    Tag(String),
    Class(String),
    Id(String),
}

pub struct SimpleSelectorParser {
    input: String,
    position: usize,
}

impl SimpleSelectorParser {
    pub fn new(input: &str) -> Self {
        SimpleSelectorParser {
            input: input.to_string(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn consume(&mut self) -> Option<char> {
        let current = self.peek();
        self.position += 1;
        current
    }

    fn consume_identifier(&mut self) -> String {
        let mut identifier = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                identifier.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        identifier
    }

    pub fn parse(&mut self) -> Option<SimpleSelector> {
        match self.peek() {
            Some('.') => {
                self.consume();
                Some(SimpleSelector::Class(self.consume_identifier()))
            }
            Some('#') => {
                self.consume();
                Some(SimpleSelector::Id(self.consume_identifier()))
            }
            Some(c) if c.is_alphabetic() => Some(SimpleSelector::Tag(self.consume_identifier())),
            _ => None,
        }
    }
}

fn find_word_in_string(text: &str, word_to_find: &str) -> bool {
    text.split_whitespace().any(|word| word == word_to_find)
}

fn find_attr(attrs: &RefCell<Vec<html5ever::Attribute>>, name: &str, value: &str) -> bool {
    attrs.borrow().iter().any(|attr| {
        std::str::from_utf8(attr.name.local.as_bytes()).map_or(false, |n| {
            name == n
                && std::str::from_utf8(attr.value.as_bytes())
                    .map_or(false, |v| find_word_in_string(v, value))
        })
    })
}

pub struct HtmlWalker<'a> {
    found_article: bool,
    pub title: String,
    pub ignore_elements: HashSet<String>,
    handle: &'a Handle,
    buffer: &'a mut String,
    selector: SimpleSelector,
}
#[derive(Clone, Copy, PartialEq)]
enum State {
    Search,
    Article,
    Title,
    Pre,
}

impl<'a> HtmlWalker<'a> {
    pub fn new(handle: &'a Handle, buffer: &'a mut String, selector: SimpleSelector) -> Self {
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
            buffer,
            selector,
        }
    }

    pub fn walk(&mut self) {
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
                State::Article => {
                    let text = contents.borrow().replace('\n', " ");
                    if !text.trim().is_empty() {
                        write!(self.buffer, "{}", text).expect("buffer overflow");
                    }
                }
                State::Title => {
                    self.title = contents.borrow().clone().trim().into();
                    //no longer needed to look into head elements
                    self.ignore_elements.insert("head".to_string());
                }
                State::Pre => {
                    write!(self.buffer, "{}", contents.borrow()).expect("buffer overflow")
                }
                State::Search => {}
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
        match state {
            State::Article => match name {
                "p" => {
                    self.walk_children(node, state);
                    writeln!(self.buffer).expect("buffer overflow");
                }
                "br" => writeln!(self.buffer).expect("buffer overflow"),
                "pre" => {
                    self.print_pre(node);
                    writeln!(self.buffer).expect("buffer overflow");
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
                _ if name.starts_with('h') && name.len() == 2 => self.walk_children(node, state),
                _ => self.walk_children(node, state),
            },
            State::Search => {
                if self.found_article {
                    return;
                } else if self.is_article(name, attrs) {
                    self.found_article = true;
                    self.walk_children(node, State::Article);
                } else {
                    match name {
                        "title" => self.walk_children(node, State::Title),
                        _ => self.walk_children(node, state),
                    }
                }
            }
            _ => self.walk_children(node, state),
        }
    }
    fn is_article(&mut self, name: &str, attrs: &RefCell<Vec<html5ever::Attribute>>) -> bool {
        match self.selector {
            SimpleSelector::Id(ref e) => find_attr(attrs, "id", e),
            SimpleSelector::Class(ref e) => find_attr(attrs, "class", e),
            SimpleSelector::Tag(ref e) => name == e,
        }
    }

    fn print_pre(&mut self, handle: &Handle) {
        for child in handle.children.borrow().iter() {
            match child.data {
                NodeData::Text { ref contents } => {
                    write!(self.buffer, "{}", contents.borrow()).expect("buffer overflow");
                }

                NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } => {
                    let name = std::str::from_utf8(name.local.as_bytes()).unwrap();
                    self.handle_element(name, attrs, child, State::Pre);
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
                    writeln!(self.buffer).expect("buffer overflow");
                } else if string == "th" || string == "td" {
                    write!(self.buffer, "\t").expect("buffer overflow");
                    self.walk_children(child, State::Article);
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
                    write!(self.buffer, "• ").expect("buffer overflow");
                    self.walk_children(child, State::Article);
                    writeln!(self.buffer).expect("buffer overflow");
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
                    write!(self.buffer, "{}. ", pos).expect("buffer overflow");
                    self.walk_children(child, State::Article);
                    writeln!(self.buffer).expect("buffer overflow");
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
