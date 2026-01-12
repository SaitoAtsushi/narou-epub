pub trait Unescape {
    fn unescape(&self) -> String;
}

impl<T: AsRef<str>> Unescape for T {
    fn unescape(&self) -> String {
        let mut newstr = String::new();
        let mut iter = self.as_ref().chars();
        'outer: while let Some(ch) = iter.next() {
            if ch == '&' {
                let mut tempstr = String::new();
                loop {
                    match iter.next() {
                        Some(';') => {
                            if tempstr == "amp" {
                                newstr.push('&');
                                break;
                            } else if tempstr == "lt" {
                                newstr.push('<');
                                break;
                            } else if tempstr == "gt" {
                                newstr.push('>');
                                break;
                            } else if tempstr == "quot" {
                                newstr.push('"');
                                break;
                            } else {
                                newstr.push('&');
                                newstr.push_str(&tempstr);
                                newstr.push(';');
                                break;
                            }
                        }
                        Some(ch) => {
                            tempstr.push(ch);
                        }
                        None => {
                            break 'outer;
                        }
                    }
                }
            } else {
                newstr.push(ch);
            }
        }
        newstr
    }
}
