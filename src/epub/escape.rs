pub trait Escape {
    fn escape(&self) -> String;
}

impl<T: AsRef<str>> Escape for T {
    fn escape(&self) -> String {
        let mut newstr = String::new();
        for ch in self.as_ref().chars() {
            match ch {
                '&' => newstr.push_str("&amp;"),
                '"' => newstr.push_str("&quot;"),
                '<' => newstr.push_str("&lt;"),
                '>' => newstr.push_str("&gt;"),
                nchar => newstr.push(nchar),
            }
        }
        newstr
    }
}
