use std::fmt::Write;
use std::marker::PhantomData;

pub trait IdPolicy: From<usize> + Into<usize> {
    const FIRST_LETTER: &[u8];
    const LETTER: &[u8];
    fn number(&self) -> usize;
}

pub struct ItemId {
    number: usize,
}

pub struct NameId {
    number: usize,
}

impl From<usize> for ItemId {
    fn from(number: usize) -> Self {
        Self { number }
    }
}

impl From<ItemId> for usize {
    fn from(id: ItemId) -> Self {
        id.number
    }
}

impl From<usize> for NameId {
    fn from(number: usize) -> Self {
        Self { number }
    }
}

impl From<NameId> for usize {
    fn from(id: NameId) -> Self {
        id.number
    }
}

impl IdPolicy for ItemId {
    const FIRST_LETTER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    const LETTER: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    fn number(&self) -> usize {
        self.number
    }
}

impl IdPolicy for NameId {
    const FIRST_LETTER: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    const LETTER: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    fn number(&self) -> usize {
        self.number
    }
}

pub struct Id<T: IdPolicy> {
    policy: T,
}

impl<T: IdPolicy> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut n = self.policy.number();
        f.write_char(T::FIRST_LETTER[n % T::FIRST_LETTER.len()].into())?;
        n /= T::FIRST_LETTER.len();
        while n != 0 {
            f.write_char(T::LETTER[n % T::LETTER.len()].into())?;
            n /= 62;
        }
        Ok(())
    }
}

pub struct IdIter<T> {
    number: usize,
    phantom: PhantomData<T>,
}

impl<T: IdPolicy> IdIter<T> {
    pub fn new() -> Self {
        IdIter {
            number: 0,
            phantom: PhantomData,
        }
    }
}

impl<T: IdPolicy> Iterator for IdIter<T> {
    type Item = Id<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let t = self.number;
        self.number += 1;
        Some(Id { policy: T::from(t) })
    }
}
