use crate::SmallString;
use crate::{chi, iota, rho_pi, theta};
use std::str;

pub union Sponge {
    pub uint64s: [u64; 25],
    pub chars: [u8; 200],
}

impl Default for Sponge {
    fn default() -> Self {
        Self {
            uint64s: [0u64; 25],
        }
    }
}

impl Sponge {
    pub unsafe fn fill(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) {
        let o = self.fill_sponge(function_name, nonce, function_params);
        self.chars[o] = 0x01;
    }

    pub unsafe fn fill_and_get_name(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> String {
        let o = self.fill_sponge(function_name, nonce, function_params);
        self.chars[o] = 0x00;

        str::from_utf8_unchecked(&self.chars[..o]).to_owned()
    }

    pub unsafe fn fill_sponge(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> usize {
        let mut offset = self.fill_sponge_single(0, function_name);
        offset += write_decimal(&mut self.chars[offset..], nonce);
        offset += self.fill_sponge_single(offset, function_params);

        let end = 200;
        self.chars[offset..end].fill(0);
        self.chars[135] = 0x80;
        offset
    }

    pub unsafe fn fill_sponge_single(&mut self, offset: usize, s: &SmallString) -> usize {
        self.chars[offset..][..s.length].copy_from_slice(&s.data[..s.length]);
        s.length
    }

    pub unsafe fn compute_selectors(&mut self) -> u32 {
        self.iters();
        self.uint64s[0] as u32
    }

    pub unsafe fn iters(&mut self) {
        let b = &mut [0u64; 5];
        [
            0x0000000000000001,
            0x0000000000008082,
            0x800000000000808a,
            0x8000000080008000,
            0x000000000000808b,
            0x0000000080000001,
            0x8000000080008081,
            0x8000000000008009,
            0x000000000000008a,
            0x0000000000000088,
            0x0000000080008009,
            0x000000008000000a,
            0x000000008000808b,
            0x800000000000008b,
            0x8000000000008089,
            0x8000000000008003,
            0x8000000000008002,
            0x8000000000000080,
            0x000000000000800a,
            0x800000008000000a,
            0x8000000080008081,
            0x8000000000008080,
            0x0000000080000001,
            0x8000000080008008,
        ]
        .into_iter()
        .for_each(|v| {
            self.iter(b, v);
        });
    }

    pub(crate) unsafe fn iter(&mut self, b: &mut [u64; 5], x: u64) {
        let a = &mut self.uint64s;
        theta(a, b);
        rho_pi(a, b);
        chi(a);
        iota(a, x);
    }
}

fn write_decimal(out: &mut [u8], mut x: u64) -> usize {
    let mut buff = [0u8; 64];
    let mid = 32;
    let mut p = mid;

    while x != 0 {
        p -= 1;
        buff[p] = (x % 10) as u8 + b'0';
        x /= 10;
    }

    let len = mid - p;
    out[..len].copy_from_slice(&buff[p..mid]);
    len
}
