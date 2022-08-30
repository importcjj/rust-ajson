mod builder;
mod parser;

mod query;
mod sub_selector;

use std::fmt;

use builder::Builder;
pub use sub_selector::SubSelector;

use self::query::{Query, DEFAULT_NONE_QUERY};
#[cfg(feature = "wild")]
use crate::wild;
use crate::{unescape, util, Result};

pub const DEFAULT_NONE_PATH: Path = Path {
    ok: false,
    part: &[],
    next: &[],
    more: false,
    #[cfg(feature = "wild")]
    wild: false,
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
    #[cfg(feature = "wild")]
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
        #[cfg(feature = "wild")]
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

    pub fn builder<'b>() -> Builder<'b> {
        Default::default()
    }

    pub fn is_match(&self, key: &[u8], key_esc: bool) -> bool {
        #[cfg(feature = "wild")]
        if self.wild {
            return wild::is_match_u8(key, self.part);
            return false;
        }

        if key_esc {
            let s = unescape(key);
            s.as_bytes().eq(self.part)
        } else if self.esc {
            util::equal_escape_u8(key, self.part)
        } else {
            key.eq(self.part)
        }
    }

    pub fn parse_next(&self) -> Result<Path<'a>> {
        if self.next.is_empty() {
            Ok(Path::default())
        } else {
            Path::from_slice(self.next)
        }
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
