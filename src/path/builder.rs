use super::{query::Query, sub_selector::SubSelector, Path};

#[derive(Default)]
pub struct Builder<'a> {
    ok:        Option<bool>,
    ident:     Option<&'a [u8]>,
    next:      Option<&'a [u8]>,
    more:      Option<bool>,
    #[cfg(feature = "wild")]
    wild:      Option<bool>,
    arrch:     Option<bool>,
    query:     Option<Query<'a>>,
    selectors: Option<Vec<SubSelector<'a>>>,
    arrsel:    Option<bool>,
    esc:       Option<bool>,
}

impl<'a> Builder<'a> {
    pub fn build(self) -> crate::Result<Path<'a>> {
        Ok(Path {
            ok: self.ok.unwrap_or_default(),
            part: self.ident.unwrap_or_default(),
            next: self.next.unwrap_or_default(),
            query: self.query,
            selectors: self.selectors,
            arrsel: self.arrsel.unwrap_or_default(),
            more: self.more.unwrap_or_default(),
            #[cfg(feature = "wild")]
            wild: self.wild.unwrap_or_default(),
            arrch: self.arrch.unwrap_or_default(),
            esc: self.esc.unwrap_or_default(),
        })
    }
}

impl<'a> Builder<'a> {
    pub fn ident(mut self, ident: &'a [u8]) -> Self {
        self.ident = Some(ident);
        self
    }

    pub fn more(mut self, more: bool) -> Self {
        self.more = Some(more);
        self
    }

    pub fn next(mut self, next: &'a [u8]) -> Self {
        self.next = Some(next);
        self
    }

    #[cfg(feature = "wild")]
    pub fn wild(mut self, wild: bool) -> Self {
        self.wild = Some(wild);
        self
    }

    pub fn ok(mut self, ok: bool) -> Self {
        self.ok = Some(ok);
        self
    }

    pub fn esc(mut self, esc: bool) -> Self {
        self.esc = Some(esc);
        self
    }

    pub fn arrch(mut self, arrch: bool) -> Self {
        self.arrch = Some(arrch);
        self
    }

    pub fn query(mut self, query: Query<'a>) -> Self {
        self.query = Some(query);
        self
    }

    pub fn selector(mut self, sels: Vec<SubSelector<'a>>) -> Self {
        self.selectors = Some(sels);
        self
    }

    pub fn arrsel(mut self, arrsel: bool) -> Self {
        self.arrsel = Some(arrsel);
        self
    }
}
