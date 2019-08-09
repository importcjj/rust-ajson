
use reader::{ByteReader, RefReader};
use wild;
use util;
use std::collections::HashMap;
use std::collections::LinkedList;
macro_rules! expect_node {
    ($s: ident, $r: ident, $start: ident, $depth: ident) => {{
        let mut end = $start;
        let mut arrch = false;
        let mut wild = false;
        let mut query = false;
        let mut finish = false;

        while let Some(b) = $r.next() {
            match b {
                b'\\' => {
                    $r.next();
                }
                b'[' | b'{' | b'(' => {
                    $depth += 1;
                    if $depth == 1 && Some(b'#') == $r.prev() {
                        query = true;
                    }
                }
                b']' | b'}' | b')' => $depth -= 1,
                b'#' => {
                    if $depth == 0 {
                        arrch = true;
                    }
                }
                b'*' | b'?' => {
                    if $depth == 0 {
                        wild = true;
                    }
                }
                b'.' => {
                    if $depth == 0 {
                        end = $r.position();
                        break;
                    }
                }
                _ => (),
            }
        }


        let mut node = if end > $start {
            Node::new(&$s[$start..end])
        } else {
            finish = true;
            Node::new(&$s[$start..])
        };

        if end > $start {
            $start = $r.position() + 1;
        }

        node.query = query;
        node.wild = wild;
        node.arrch = arrch;

        (node, finish)
    }};
}

pub type Path<'a> = LinkedList<Node<'a>>;

#[derive(Debug)]
pub struct Node<'a> {
    pub raw: &'a str,
    pub query: bool,
    pub wild: bool,
    pub arrch: bool,
}

#[derive(Debug)]
pub struct Group<'a> {
    pub index: usize,
    pub raw: &'a str,
    pub wild: bool,
    pub path_end: bool,
    pub sub_count: usize,
    pub match_count: usize,
    pub m: Option<Vec<Group<'a>>>,
}

impl<'a> Group<'a> {
    pub fn new(s: &'a str) -> Group<'a> {
        Group {
            index: 0,
            raw: s,
            wild: false,
            path_end: false,
            m: None,
            sub_count: 0,
            match_count: 0,
        }
    }

    pub fn has_sub_path(&self) -> bool {
        self.m.is_some()
    }

    pub fn match_key(&self, k: &[u8]) -> bool {
        if self.wild {
            wild::is_match_u8(k, self.raw.as_bytes())
        } else {
            util::equal_escape_u8(k, self.raw.as_bytes())
        }
    }

    pub fn add(&mut self, i: usize, mut p: Path<'a>) {
        let node = p.pop_front();
        match node {
            Some(n) => {
                if self.m.is_none() {
                    self.m = Some(Vec::new());
                }
                // if self.m.is_none() {
                //     self.m = Some(HashMap::new());
                // }
                
                // let mut group = self.m.as_mut().unwrap().entry(n.raw).or_insert({
                //     self.sub_count += 1;
                //     let g = Group::new(n.raw);
                //     g.wild = n.wild;
                //     g
                // });

                // group.add(i, p);
                // return

                for group in self.m.as_mut().unwrap().iter_mut() {
                    if group.raw == n.raw {
                        group.add(i, p);
                        return
                    }
                }
                let mut group = Group::new(n.raw);
                group.wild = n.wild;
                group.add(i, p);
                self.m.as_mut().unwrap().push(group);
                self.sub_count += 1;
            }
            None => {
                self.path_end = true;
                self.index = i;
            },
        }

    }

    pub fn need_match(&self) -> bool {
        self.match_count < self.sub_count
    }


    pub fn find_sub_group(&mut self, key: &[u8]) -> Option<&mut Group<'a>> {
        for group in self.m.as_mut().unwrap().iter_mut() {
            if group.match_key(key) {
                self.match_count += 1;
                return Some(group)
            }
        }

        None
    }
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
    let mut r = RefReader::new(s.as_bytes());
    let mut start = 0;
    let mut depth = 0;

    // let mut arrch = false;
    // let mut wild = false;
    // let mut query = false;
    loop {
        let (node, finish) = expect_node!(s, r, start, depth);
        path.push_back(node);
        if finish {
            break;
        }
    }

    // while let Some(b) = r.next() {
    //     match b {
    //         b'\\' => {
    //             r.next();
    //         }
    //         b'[' | b'{' | b'(' => {
    //             depth += 1;
    //             if depth == 1 && Some(b'#') == r.prev() {
    //                 query = true;
    //             }
    //         }
    //         b']' | b'}' | b')' => depth -= 1,
    //         b'#' => {
    //             if depth == 0 {
    //                 arrch = true;
    //             }
    //         }
    //         b'*' | b'?' => {
    //             if depth == 0 {
    //                 wild = true;
    //             }
    //         }
    //         b'.' => {
    //             if depth == 0 {
    //                 let mut node = Node::new(&s[start..r.position()]);

    //                 node.query = query;
    //                 node.wild = wild;
    //                 node.arrch = arrch;

    //                 query = false;
    //                 wild = false;
    //                 arrch = false;

    //                 path.push_back(node);
    //                 start = r.position() + 1;
    //             }
    //         }
    //         _ => continue,
    //     }
    // }

    // if r.position() > start {
    //     let node = Node::new(&s[start..]);
    //     path.push_back(node);
    // }

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


    #[test]
    fn test_path_v2_group() {
        let mut group = Group::new("");
        for (i, s) in vec![
            "widget.window.name",
            "widget.image.hOffset",
            "widget.text.onMouseUp",
            "widget.debug",
            "widget.menu.#(sub_item>7)#.title",
            "widget.menu.#(nets.#(==7))#.title",
            "overflow"
        ].iter().enumerate() {
            let path = parse(s).unwrap();
            group.add(i, path);

        }

        println!("{:?}", group);
    }
}