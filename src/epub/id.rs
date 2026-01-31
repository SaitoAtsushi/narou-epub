pub struct Id {
    first_letter: &'static [u8],
    letter: &'static [u8],
    number: usize,
}

impl Id {
    pub fn new_for_name() -> Self {
        Self {
            first_letter: b"0123456789abcdefghijklmnopqrstuvwxyz",
            letter: b"0123456789abcdefghijklmnopqrstuvwxyz",
            number: 0,
        }
    }

    pub fn new_for_id() -> Self {
        Self {
            first_letter: b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            letter: b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            number: 0,
        }
    }
}

impl Iterator for Id {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let mut n = self.number;
        let mut newstr = String::new();
        newstr.push(self.first_letter[n % self.first_letter.len()].into());
        n /= self.first_letter.len();
        while n != 0 {
            newstr.push(self.letter[n % self.letter.len()].into());
            n /= self.letter.len();
        }
        self.number += 1;
        Some(newstr)
    }
}
