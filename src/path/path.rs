use std::fmt;

use super::{
    parser,
    query::{Query, DEFAULT_NONE_QUERY},
    sub_selector::SubSelector,
};
#[cfg(feature = "wild")]
use crate::wild;
use crate::{unescape, util, Result};

pub const DEFAULT_NONE_PATH: Path = Path {
    ok:    false,
    part:  &[],
    next:  &[],
    more:  false,
    wild:  false,
    arrch: false,

    query:     None,
    selectors: None,
    arrsel:    false,
    esc:       false,
};

#[derive(Default)]
pub struct Path<'a> {
    pub ok:        bool,
    pub part:      &'a [u8],
    pub next:      &'a [u8],
    pub query:     Option<Query<'a>>,
    pub selectors: Option<Vec<SubSelector<'a>>>,
    pub arrsel:    bool,
    pub more:      bool,
    pub wild:      bool,
    pub arrch:     bool,
    pub esc:       bool,
}

impl<'a> fmt::Debug for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Path")?;
        write!(f, " ok={}", self.ok)?;
        write!(f, " part=`{:?}`", unsafe {
            std::str::from_utf8_unchecked(self.part)
        })?;

        write!(f, " more={}", self.more)?;
        write!(f, " wild={}", self.wild)?;
        write!(f, " arrch={}", self.arrch)?;

        if self.selectors.is_some() {
            for sel in self.borrow_selectors() {
                write!(f, "\n\tselector {:?}", sel)?;
            }
        }
        if self.has_query() {
            write!(f, " query={:?}", self.query)?;
        }
        write!(f, ">")
    }
}

impl<'a> Path<'a> {
    pub fn from_slice(v: &'a [u8]) -> Result<Path<'a>> {
        parser::parse(v)
    }

    pub fn is_match(&self, key: &[u8], key_esc: bool) -> bool {
        // let optional_key = if key.contains(&b'\\') {
        //     Some(unescape(key))
        // } else {
        //     None
        // };
        // let key = optional_key.as_ref().map_or(key, |v| v.as_bytes());
        if self.wild {
            #[cfg(feature = "wild")]
            return wild::is_match_u8(key, self.part);
            false
        } else if key_esc {
            let s = unescape(key);
            s.as_bytes().eq(self.part)
        } else if self.esc {
            util::equal_escape_u8(key, self.part)
        } else {
            key.eq(self.part)
        }
    }

    pub fn set_part(&mut self, v: &'a [u8]) {
        self.part = v;
    }

    pub fn set_more(&mut self, b: bool) {
        self.more = b;
    }

    pub fn set_next(&mut self, next: &'a [u8]) {
        self.next = next;
    }

    pub fn parse_next(&self) -> Result<Path<'a>> {
        if self.next.is_empty() {
            Ok(Path::default())
        } else {
            Path::from_slice(self.next)
        }
    }

    #[cfg(feature = "wild")]
    pub fn set_wild(&mut self, b: bool) {
        self.wild = b;
    }

    pub fn set_ok(&mut self, b: bool) {
        self.ok = b;
    }

    pub fn set_arrch(&mut self, b: bool) {
        self.arrch = b;
    }

    pub fn set_q(&mut self, q: Query<'a>) {
        self.query = Some(q);
    }

    pub fn set_esc(&mut self, b: bool) {
        self.esc = b;
    }

    pub fn has_query(&self) -> bool {
        self.query.is_some()
    }

    pub fn borrow_query(&self) -> &Query<'a> {
        match self.query {
            Some(_) => self.query.as_ref().unwrap(),
            None => &DEFAULT_NONE_QUERY,
        }
    }

    pub fn set_selectors(&mut self, selectors: Vec<SubSelector<'a>>) {
        self.selectors = Some(selectors);
    }

    pub fn set_arrsel(&mut self, b: bool) {
        self.arrsel = b;
    }

    pub fn has_selectors(&self) -> bool {
        self.selectors.is_some()
    }

    pub fn borrow_selectors(&self) -> &[SubSelector<'a>] {
        match self.selectors {
            Some(_) => self.selectors.as_ref().unwrap(),
            None => &[],
        }
    }
}
