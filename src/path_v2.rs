
use reader::{ByteReader, StrReader};
use std::collections::LinkedList;
pub type Path<'a> = LinkedList<Node<'a>>;

#[derive(Debug)]
pub struct Node<'a> {
    pub raw: &'a str,
    pub query: bool,
    pub wild: bool,
    pub arrch: bool,
}


impl<'a> Node<'a> {
    pub fn new(v: &str) -> Node {
        Node {
            raw: v,
            query: false,
            wild: false,
            arrch: false,
        }
    }
}

pub fn parse(s: &str) -> Option<Path> {
    let mut path = Path::new();
    let mut r = StrReader::new(s);
    let mut start = 0;
    let mut depth = 0;

    let mut arrch = false;
    let mut wild = false;
    let mut query = false;

    while let Some(b) = r.next() {
        match b {
            b'\\' => {
                r.next();
            }
            b'[' | b'{' | b'(' => {
                depth += 1;
                if depth == 1 && Some(b'#') == r.prev() {
                    query = true;
                }
            }
            b']' | b'}' | b')' => depth -= 1,
            b'#' => {
                if depth == 0 {
                    arrch = true;
                }
            }
            b'*' | b'?' => {
                if depth == 0 {
                    wild = true;
                }
            }
            b'.' => {
                if depth == 0 {
                    let mut node = Node::new(&s[start..r.position()]);
                    
                    node.query = query;
                    node.wild = wild;
                    node.arrch = arrch;

                    query = false;
                    wild = false;
                    arrch = false;

                    path.push_back(node);
                    start = r.position() + 1;
                }
            }
            _ => continue,
        }
    }

    if r.position() > start {
        let node = Node::new(&s[start..]);
        path.push_back(node);
    }

    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_v2_parse() {
        for s in vec![
            "widget.window.name",
            "widget.image.hOffset",
            "widget.text.onMouseUp",
            "widget.debug",
            "widget.menu.#(sub_item>7)#.title",
            "widget.menu.#(nets.#(==7))#.title",
        ] {
            let path = parse(s);
            println!("{:?}", path);
        }
    }
}