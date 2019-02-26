use std::num::Wrapping;

#[derive(Clone)]
pub struct Bitset(Vec<u64>);

impl Bitset {
    pub fn new(bits: usize) -> Self {
        let mut extra = 0;
        if bits % 64 != 0 {
            extra = 1;
        }
        Bitset(vec![0; bits / 64 + extra])
    }

    pub fn set(&mut self, pos: usize) {
        let (major, minor) = bitset_index(pos);
        self.0[major] |= 1 << minor;
    }

    pub fn clear(&mut self, pos: usize) {
        let (major, minor) = bitset_index(pos);
        self.0[major] &= !(1 << minor);
    }

    pub fn get(&self, pos: usize) -> bool {
        let (major, minor) = bitset_index(pos);
        self.0[major] & (1 << minor) != 0
    }

    fn popcnt(&self) -> usize {
        let mut total = 0;
        for b in &self.0 {
            let mut v = b.clone();
            v = (v & 0x5555555555555555) + ((v & 0xAAAAAAAAAAAAAAAA) >> 1);
            v = (v & 0x3333333333333333) + ((v & 0xCCCCCCCCCCCCCCCC) >> 2);
            v = (v & 0x0F0F0F0F0F0F0F0F) + ((v & 0xF0F0F0F0F0F0F0F0) >> 4);
            v = (Wrapping(v) * Wrapping(0x0101010101010101)).0;
            total += ((v >> 56) & 0xFF) as usize;
        }
        total
    }

    pub fn hash(&self) -> u64 {
        let mut hash = self.popcnt() as u64;
        for v in &self.0 {
            hash ^= v;
        }
        hash
    }

    pub fn equals(&self, b2: &Bitset) -> bool {
        let b = &self.0;
        let b2 = &b2.0;
        if b.len() != b2.len() {
            return false;
        }
        for i in 0..b.len() {
            if b[i] != b2[i] {
                return false;
            }
        }
        true
    }
}

fn bitset_index(pos: usize) -> (usize, usize) {
    (pos / 64, pos % 64)
}
