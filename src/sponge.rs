use crate::SmallString;

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
    /// # Safety
    ///
    /// This function is unsafe because it writes to a union type.
    pub unsafe fn fill<const N: usize>(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) {
        let o = self.fill_sponge::<N>(function_name, nonce, function_params);
        self.chars[o] = 0x01;
    }

    /// # Safety
    ///
    /// This function is unsafe because it writes to a union type.
    pub unsafe fn fill_and_get_name<const N: usize>(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> String {
        let o = self.fill_sponge::<N>(function_name, nonce, function_params);
        self.chars[o] = 0x00;

        str::from_utf8_unchecked(&self.chars[..o]).to_owned()
    }

    /// # Safety
    ///
    /// This function is unsafe because it writes to a union type.
    pub unsafe fn fill_sponge<const N: usize>(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> usize {
        let mut offset = self.fill_sponge_single::<N>(0, function_name);
        offset += write_decimal(&mut self.chars[offset..], nonce);
        offset += self.fill_sponge_single::<N>(offset, function_params);

        self.chars[offset..135].fill(0);
        self.chars[135] = 0x80;
        self.chars[136..200].fill(0);
        offset
    }

    /// # Safety
    ///
    /// This function is unsafe because it writes to a union type.
    pub unsafe fn fill_sponge_single<const N: usize>(
        &mut self,
        offset: usize,
        s: &SmallString,
    ) -> usize {
        let n = if N == 0 { s.length } else { N };
        self.chars[offset..][..n].copy_from_slice(&s.data[..n]);
        s.length
    }

    /// # Safety
    ///
    /// This function is unsafe because it uses SIMD instructions and a union type.
    pub unsafe fn compute_selectors(&mut self) -> u32 {
        crate::iters(&mut self.uint64s);
        self.uint64s[0] as u32
    }
}

fn write_decimal(out: &mut [u8], mut x: u64) -> usize {
    let mut buff = [0u8; 32];
    let n = 32;
    let mut p = n;

    while x != 0 {
        p -= 1;
        buff[p] = (x % 10) as u8 + b'0';
        x /= 10;
    }

    let len = n - p;
    out[..len].copy_from_slice(&buff[p..]);
    len
}
