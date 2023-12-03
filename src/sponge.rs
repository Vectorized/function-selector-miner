use crate::SmallString;

use std::str;

pub union Sponge {
    pub uint64s: [u64; 25],
    pub chars: [u8; 200],
}

impl Default for Sponge {
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    pub unsafe fn fill_sponge<const N: usize>(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> usize {
        let mut offset = self.fill_sponge_single::<N>(0, function_name);
        offset += write_decimal::<N>(&mut self.chars[offset..], nonce);
        offset += self.fill_sponge_single::<N>(offset, function_params);

        self.chars[offset..135].fill(0);
        self.chars[135] = 0x80;
        offset
    }

    /// # Safety
    ///
    /// This function is unsafe because it writes to a union type.
    #[inline(always)]
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
    #[inline(always)]
    pub unsafe fn compute_selectors(&mut self) -> u32 {
        crate::iters(&mut self.uint64s);
        self.uint64s[0] as u32
    }
}

static DEC_DIGITS_LUT: &[u8; 200] = b"\
0001020304050607080910111213141516171819\
2021222324252627282930313233343536373839\
4041424344454647484950515253545556575859\
6061626364656667686970717273747576777879\
8081828384858687888990919293949596979899";

/// # Safety
///
/// This function is unsafe because it uses `copy_nonoverlapping`.
#[inline(always)]
unsafe fn write_decimal<const N: usize>(out: &mut [u8], mut x: u64) -> usize {
    let mut buf = [0u8; 64];

    let buf_ptr = if N == 0 {
        buf.as_mut_ptr()
    } else {
        out.as_mut_ptr().add(32)
    };
    let lut_ptr = DEC_DIGITS_LUT.as_ptr();

    let mut curr = 32;

    while x >= 10000 {
        let rem = (x % 10000) as usize;
        x /= 10000;

        let d1 = (rem / 100) << 1;
        let d2 = (rem % 100) << 1;
        curr -= 4;

        std::ptr::copy_nonoverlapping(lut_ptr.add(d1), buf_ptr.add(curr), 2);
        std::ptr::copy_nonoverlapping(lut_ptr.add(d2), buf_ptr.add(curr + 2), 2);
    }

    let mut x = x as usize;

    if x >= 100 {
        let d1 = (x % 100) << 1;
        x /= 100;
        curr -= 2;
        std::ptr::copy_nonoverlapping(lut_ptr.add(d1), buf_ptr.add(curr), 2);
    }

    if x < 10 {
        curr -= 1;
        *buf_ptr.add(curr) = (x as u8) + b'0';
    } else {
        let d1 = x << 1;
        curr -= 2;
        std::ptr::copy_nonoverlapping(lut_ptr.add(d1), buf_ptr.add(curr), 2);
    }

    std::ptr::copy_nonoverlapping(buf_ptr.add(curr), out.as_mut_ptr(), 32);
    32 - curr
}
