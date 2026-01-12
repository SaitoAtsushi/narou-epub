use std::fmt::Write;

pub struct ItemId {
    number: u32,
}

impl ItemId {
    const FIRST_LETTER: &[u8; 52] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    const LETTER: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
}

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut n = self.number as usize;
        f.write_char(Self::FIRST_LETTER[n % Self::FIRST_LETTER.len()].into())?;
        n /= 52;
        while n != 0 {
            f.write_char(Self::LETTER[n % Self::LETTER.len()].into())?;
            n /= 62;
        }
        Ok(())
    }
}

pub struct IdIter {
    number: u32,
}

impl IdIter {
    pub fn new() -> Self {
        IdIter { number: 0 }
    }
}

impl Iterator for IdIter {
    type Item = ItemId;
    fn next(&mut self) -> Option<Self::Item> {
        let t = self.number;
        self.number += 1;
        Some(ItemId { number: t })
    }
}
