pub struct ItemId {
    number: usize,
}

impl ItemId {
    pub fn new() -> Self {
        Self { number: 0 }
    }
}

pub struct NameId {
    number: usize,
}

impl NameId {
    pub fn new() -> Self {
        Self { number: 0 }
    }
}

trait IdIter {
    const FIRST_LETTER: &[u8];
    const LETTER: &[u8];
    fn number(&mut self) -> usize;
    fn next_id(&mut self) -> String {
        let mut n = self.number();
        let mut newstr = String::new();
        newstr.push(Self::FIRST_LETTER[n % Self::FIRST_LETTER.len()].into());
        n /= Self::FIRST_LETTER.len();
        while n != 0 {
            newstr.push(Self::LETTER[n % Self::LETTER.len()].into());
            n /= Self::LETTER.len();
        }
        newstr
    }
}

impl IdIter for NameId {
    const FIRST_LETTER: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    const LETTER: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    fn number(&mut self) -> usize {
        let t = self.number;
        self.number += 1;
        t
    }
}

impl Iterator for NameId {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_id())
    }
}

impl IdIter for ItemId {
    const FIRST_LETTER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    const LETTER: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    fn number(&mut self) -> usize {
        let t = self.number;
        self.number += 1;
        t
    }
}

impl Iterator for ItemId {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_id())
    }
}
