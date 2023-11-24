use crate::SmallString;
use crate::{chi, iota, rho_pi, theta};

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
    pub unsafe fn fill_sponge(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> usize {
        let mut o = Self::fill_sponge_single(&mut self.chars, function_name);
        o += write_decimal(&mut self.chars[o..], nonce);
        o += Self::fill_sponge_single(&mut self.chars[o..], function_params);

        let end = 200;
        self.chars[o..end].fill(0);
        self.chars[135] = 0x80;

        o
    }

    pub fn fill_sponge_single(sponge: &mut [u8], s: &SmallString) -> usize {
        sponge[..s.length].copy_from_slice(&s.data[..s.length]);
        s.length
    }

    pub unsafe fn compute_selectors(&mut self) -> u32 {
        let mut b = [0u64; 5];
        self.iters(&mut b);
        self.uint64s[0] as u32
    }

    pub unsafe fn iters(&mut self, b: &mut [u64; 5]) {
        self.iter(b, 0x0000000000000001);
        self.iter(b, 0x0000000000008082);
        self.iter(b, 0x800000000000808a);
        self.iter(b, 0x8000000080008000);
        self.iter(b, 0x000000000000808b);
        self.iter(b, 0x0000000080000001);
        self.iter(b, 0x8000000080008081);
        self.iter(b, 0x8000000000008009);
        self.iter(b, 0x000000000000008a);
        self.iter(b, 0x0000000000000088);
        self.iter(b, 0x0000000080008009);
        self.iter(b, 0x000000008000000a);
        self.iter(b, 0x000000008000808b);
        self.iter(b, 0x800000000000008b);
        self.iter(b, 0x8000000000008089);
        self.iter(b, 0x8000000000008003);
        self.iter(b, 0x8000000000008002);
        self.iter(b, 0x8000000000000080);
        self.iter(b, 0x000000000000800a);
        self.iter(b, 0x800000008000000a);
        self.iter(b, 0x8000000080008081);
        self.iter(b, 0x8000000000008080);
        self.iter(b, 0x0000000080000001);
        self.iter(b, 0x8000000080008008);
    }

    unsafe fn iter(&mut self, b: &mut [u64; 5], x: u64) {
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
