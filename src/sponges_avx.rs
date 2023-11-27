use std::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_extract_epi64, _mm256_or_si256, _mm256_set1_epi64x,
    _mm256_set_epi64x, _mm256_sll_epi64, _mm256_srl_epi64, _mm256_xor_si256, _mm_set1_epi64x,
};
use std::ops::{BitAnd, BitOr, BitXor, BitXorAssign, Not, Shl, Shr};

use crate::{SmallString, Sponge};

pub struct SpongesAvx {
    compute_slices: [SpongeComputeSlice; 25],
}

#[derive(Clone, Copy, Debug)]
pub struct SpongeComputeSlice(__m256i);

impl SpongeComputeSlice {
    pub fn vals(&self) -> [u64; 4] {
        [
            unsafe { _mm256_extract_epi64(self.0, 0) as u64 },
            unsafe { _mm256_extract_epi64(self.0, 1) as u64 },
            unsafe { _mm256_extract_epi64(self.0, 2) as u64 },
            unsafe { _mm256_extract_epi64(self.0, 3) as u64 },
        ]
    }
}

impl BitOr for SpongeComputeSlice {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_or_si256(self.0, rhs.0) })
    }
}

impl BitXor<Self> for SpongeComputeSlice {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_xor_si256(self.0, rhs.0) })
    }
}

impl BitXorAssign<u64> for SpongeComputeSlice {
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 = unsafe { _mm256_xor_si256(self.0, _mm256_set1_epi64x(rhs as i64)) };
    }
}

impl BitXorAssign for SpongeComputeSlice {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 = unsafe { _mm256_xor_si256(self.0, rhs.0) };
    }
}

impl BitAnd for SpongeComputeSlice {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(unsafe { _mm256_and_si256(self.0, rhs.0) })
    }
}

impl Not for SpongeComputeSlice {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(unsafe { _mm256_xor_si256(self.0, _mm256_set1_epi64x(-1)) })
    }
}

impl Shr<u32> for SpongeComputeSlice {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        Self(unsafe { _mm256_srl_epi64(self.0, _mm_set1_epi64x(rhs as i64)) })
    }
}

impl Shl<u32> for SpongeComputeSlice {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        Self(unsafe { _mm256_sll_epi64(self.0, _mm_set1_epi64x(rhs as i64)) })
    }
}

impl Default for SpongeComputeSlice {
    fn default() -> Self {
        Self(unsafe { _mm256_set_epi64x(0, 0, 0, 0) })
    }
}

impl SpongesAvx {
    /// # Safety
    ///
    /// This function is unsafe because it uses SIMD instructions and a union type.
    pub unsafe fn new<const N: usize>(
        function_name: &SmallString,
        nonce: u64,
        function_params: &SmallString,
    ) -> Self {
        let mut sponges = <[Sponge; 4]>::default();
        sponges.iter_mut().enumerate().for_each(|(idx, sponge)| {
            sponge.fill::<N>(function_name, nonce + idx as u64, function_params)
        });

        let compute_slices = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24,
        ]
        .map(|idx| {
            SpongeComputeSlice(_mm256_set_epi64x(
                sponges[3].uint64s[idx] as i64,
                sponges[2].uint64s[idx] as i64,
                sponges[1].uint64s[idx] as i64,
                sponges[0].uint64s[idx] as i64,
            ))
        });

        Self { compute_slices }
    }

    /// # Safety
    ///
    /// This function is unsafe because it uses SIMD instructions and a union type.
    pub unsafe fn compute_selectors(&mut self) -> [u32; 4] {
        crate::iters(&mut self.compute_slices);

        let first_slice = self.compute_slices[0].0;

        [
            _mm256_extract_epi64(first_slice, 0) as u32,
            _mm256_extract_epi64(first_slice, 1) as u32,
            _mm256_extract_epi64(first_slice, 2) as u32,
            _mm256_extract_epi64(first_slice, 3) as u32,
        ]
    }
}

#[test]
fn equivalent() {
    unsafe {
        let mut s = Sponge::default();

        let function_name = SmallString::new("foo");
        let function_params = SmallString::new("foo");
        s.fill::<0>(&function_name, 0, &function_params);
        let mut s_avx = SpongesAvx::new(&function_name, 0, &function_params);
        assert_eq!(s.uint64s[0], s_avx.compute_slices[0].vals()[0]);

        let b = &mut [0u64; 5];
        let b_avx = &mut [SpongeComputeSlice::default(); 5];
        assert_eq!(b[0], b_avx[0].vals()[0]);

        const X1: u64 = 0x0000000000000001;
        const X2: u64 = 0x0000000000008082;

        crate::iter(&mut s.uint64s, b, X1);
        crate::iter(&mut s_avx.compute_slices, b_avx, X1);
        assert_eq!(b[0], b_avx[0].vals()[0]);

        crate::iter(&mut s.uint64s, b, X2);
        crate::iter(&mut s_avx.compute_slices, b_avx, X2);
        assert_eq!(b[0], b_avx[0].vals()[0]);

        let c0 = s.compute_selectors();
        let c1 = s_avx.compute_selectors();
        println!("c0: {:X?}", c0);
        println!("c1: {:X?}", c1);
        assert_eq!(c0, c1[0], "compute_selectors() failed");
        assert_eq!(c0, 0x67E41DE3)
    }
}

#[test]
fn ops() {
    let mut s = SpongeComputeSlice::default();
    println!("s0: {s:X?}");

    s ^= u64::MAX;
    println!("s0: {s:X?}");
    assert_eq!(s.vals()[0], u64::MAX);

    s = s << 4;
    println!("s0: {s:X?}");
    let expected = u64::MAX << 4;
    assert_eq!(s.vals()[0], expected);

    s = !s;
    println!("s0: {s:X?}");
    let expected = !(u64::MAX << 4);
    assert_eq!(s.vals()[0], expected);

    let t = crate::rotate_left(s, 4);
    println!("t: {t:X?}");
    let expected = crate::rotate_left(expected, 4);
    assert_eq!(t.vals()[0], expected);
}
