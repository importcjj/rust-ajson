#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}



struct Path {
    next: Option<Box<Path>>
}

impl Path {
    fn get_next(&self) -> &Path {
        self.next.as_ref().unwrap()
    }
}