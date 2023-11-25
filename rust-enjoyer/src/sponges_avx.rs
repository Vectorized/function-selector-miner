use std::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_extract_epi64, _mm256_or_si256,
    _mm256_set1_epi64x, _mm256_set_epi64x, _mm256_sll_epi64, _mm256_srl_epi64, _mm256_xor_si256,
    _mm_set1_epi64x,
};
use std::ops::{BitAnd, BitOr, BitXor, BitXorAssign, Not, Shl, Shr};

use crate::{chi, iota, rho_pi, theta};
use crate::{SmallString, Sponge};

#[derive(Default)]
pub struct SpongesAvx {
    sponges: [Sponge; 4],
    compute_slices: [SpongeComputeSlice; 25],
}

#[derive(Clone, Copy, Debug)]
pub struct SpongeComputeSlice {
    pub slice: __m256i,
}

impl SpongeComputeSlice {
    pub fn vals(&self) -> [u64; 4] {
        [
            unsafe { _mm256_extract_epi64(self.slice, 0) as u64 },
            unsafe { _mm256_extract_epi64(self.slice, 1) as u64 },
            unsafe { _mm256_extract_epi64(self.slice, 2) as u64 },
            unsafe { _mm256_extract_epi64(self.slice, 3) as u64 },
        ]
    }
}

impl BitOr for SpongeComputeSlice {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            slice: unsafe { _mm256_or_si256(self.slice, rhs.slice) },
        }
    }
}

impl BitXor<Self> for SpongeComputeSlice {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            slice: unsafe { _mm256_xor_si256(self.slice, rhs.slice) },
        }
    }
}

impl BitXorAssign<u64> for SpongeComputeSlice {
    fn bitxor_assign(&mut self, rhs: u64) {
        self.slice = unsafe { _mm256_xor_si256(self.slice, _mm256_set1_epi64x(rhs as i64)) };
    }
}

impl BitXorAssign for SpongeComputeSlice {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.slice = unsafe { _mm256_xor_si256(self.slice, rhs.slice) };
    }
}

impl BitAnd for SpongeComputeSlice {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            slice: unsafe { _mm256_and_si256(self.slice, rhs.slice) },
        }
    }
}

impl Not for SpongeComputeSlice {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            slice: unsafe {
                _mm256_xor_si256(
                    self.slice,
                    _mm256_set1_epi64x(0xffffffffffffffff_u64 as i64),
                )
            },
        }
    }
}

impl Shr<u32> for SpongeComputeSlice {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        Self {
            slice: unsafe { _mm256_srl_epi64(self.slice, _mm_set1_epi64x(rhs as i64)) },
        }
    }
}

impl Shl<u32> for SpongeComputeSlice {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        Self {
            slice: unsafe { _mm256_sll_epi64(self.slice, _mm_set1_epi64x(rhs as i64)) },
        }
    }
}

impl Default for SpongeComputeSlice {
    fn default() -> Self {
        Self {
            slice: unsafe { _mm256_set_epi64x(0, 0, 0, 0) },
        }
    }
}

impl SpongesAvx {
    pub unsafe fn fill(
        &mut self,
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) {
        self.sponges
            .iter_mut()
            .enumerate()
            .for_each(|(idx, sponge)| {
                sponge.fill(function_name, nonce + idx as u64, function_params)
            });

        // turn the 4 sponges into compute slices using the 25 uint64s
        for (idx, slice) in self.compute_slices.iter_mut().enumerate() {
            slice.slice = _mm256_set_epi64x(
                self.sponges[3].uint64s[idx] as i64,
                self.sponges[2].uint64s[idx] as i64,
                self.sponges[1].uint64s[idx] as i64,
                self.sponges[0].uint64s[idx] as i64,
            );
        }
    }

    pub unsafe fn compute_selectors(&mut self) -> [u32; 4] {
        self.iters();

        let first_slice = self.compute_slices[0].slice;
        [
            _mm256_extract_epi64(first_slice, 0) as u32,
            _mm256_extract_epi64(first_slice, 1) as u32,
            _mm256_extract_epi64(first_slice, 2) as u32,
            _mm256_extract_epi64(first_slice, 3) as u32,
        ]
    }

    unsafe fn iters(&mut self) {
        let b: &mut [SpongeComputeSlice; 5] = &mut Default::default();
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

    pub unsafe fn iter(&mut self, b: &mut [SpongeComputeSlice; 5], x: u64) {
        let a = &mut self.compute_slices;
        theta(a, b);
        rho_pi(a, b);
        chi(a);
        iota(a, x);
    }
}

#[test]
fn equivalent() {
    unsafe {
        let mut s = Sponge::default();
        let mut s_avx = SpongesAvx::default();

        let function_name = SmallString::new("someFunction");
        let function_params = SmallString::new("(uint256,address)");
        assert_eq!(s.chars, s_avx.sponges[0].chars);

        s.fill(&function_name, 0, &function_params);
        s_avx.fill(&function_name, 0, &function_params);
        assert_eq!(s.chars, s_avx.sponges[0].chars);

        let b = &mut [0u64; 5];
        let b_avx = &mut [SpongeComputeSlice::default(); 5];
        assert_eq!(b[0], b_avx[0].vals()[0]);

        const x1: u64 = 0x0000000000000001;
        const x2: u64 = 0x0000000000008082;

        s.iter(b, x1);
        s_avx.iter(b_avx, x1);
        assert_eq!(b[0], b_avx[0].vals()[0]);

        s.iter(b, x2);
        s_avx.iter(b_avx, x2);
        assert_eq!(b[0], b_avx[0].vals()[0]);

        let c0 = unsafe { s.compute_selectors() };
        let c1 = unsafe { s_avx.compute_selectors() };
        assert_eq!(c0, c1[0], "compute_selectors() failed");
    }
}

#[test]
fn ops() {
    let mut s0 = SpongeComputeSlice::default();

    println!("s0: {:X?}", s0);

    s0 ^= u64::MAX;
    s0 = s0 << 4;

    s0 = !s0;
    println!("s0: {:X?}", s0);

    let t = crate::rotate_left(s0, 4);
    println!("t: {:X?}", t);
}
