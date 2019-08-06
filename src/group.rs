
use path::Path;
use path_parser::new_path_from_utf8;
use std::collections::LinkedList;


#[derive(Debug)]
pub struct PathGroup<'a> {
    end: bool,
    part: &'a [u8],
    path: Option<Path<'a>>,
    val_index: usize,
    children: LinkedList<PathGroup<'a>>, // children: HashMap<&'a [u8], PathGroup<'a>>
}

impl<'a> PathGroup<'a> {
    pub fn new() -> PathGroup<'a> {
        PathGroup {
            end: false,
            val_index: 0,
            part: &[],
            path: None,
            children: LinkedList::new(),
        }
    }

    pub fn push_path(&mut self, val_index: usize, mut path: Path<'a>) {
        for g in self.children.iter_mut() {
            if g.part == path.part {
                if !path.more {
                    g.end = true;
                    g.val_index = val_index;
                } else {
                    // let next = path.take_next();
                    g.push_path(val_index, *path.next.take().unwrap());
                }

                return;
            }
        }

        let mut g = PathGroup::new();
        g.part = path.part;

        if !path.more {
            g.end = true;
            g.val_index = val_index;
        } else {
            g.push_path(val_index, *path.next.take().unwrap());
        }

        g.path = Some(path);
        self.children.push_front(g);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_path() {
        let mut g = PathGroup::new();
        let pathes = vec![
            "widget.window.name",
            "widget.image.hOffset",
            "widget.text.onMouseUp",
            "widget.debug",
            "widget.menu.#(sub_item>7)#.title",
        ];
        for (i, s) in pathes.iter().enumerate() {
            let path = Path::new_from_utf8(s.as_bytes());
            g.push_path(i, path);
        }
        println!("{:?}", g);
    }

    #[test]
    fn test_multi_path() {
        let mut group = PathGroup::new();
        for (i, s) in vec!["widgent.window.title", "widget.image.hoffset"]
            .iter()
            .enumerate()
        {
            let path = Path::new_from_utf8(s.as_bytes());
            group.push_path(i, path);
        }
    }
}