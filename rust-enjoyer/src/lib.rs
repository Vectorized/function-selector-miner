pub mod small_string;
pub mod sponge;
pub mod sponges_avx;

pub use small_string::*;
pub use sponge::*;

use std::arch::x86_64::*;
use std::ops::{BitAnd, BitOr, BitXor, BitXorAssign, Not, Shl, Shr};

fn theta_<T>(a: &mut [T], b: &[T], m: usize, n: usize, o: usize)
where
    T: BitXorAssign
        + BitXor<Output = T>
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + BitOr<Output = T>
        + Copy,
{
    let t = b[m] ^ rotate_left(b[n], 1);
    a[o] ^= t;
    a[o + 5] ^= t;
    a[o + 10] ^= t;
    a[o + 15] ^= t;
    a[o + 20] ^= t;
}

pub fn theta<T>(a: &mut [T], b: &mut [T])
where
    T: BitXorAssign
        + BitXor<Output = T>
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + BitOr<Output = T>
        + Copy,
{
    b[0] = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20];
    b[1] = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21];
    b[2] = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22];
    b[3] = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23];
    b[4] = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24];

    theta_(a, b, 4, 1, 0);
    theta_(a, b, 0, 2, 1);
    theta_(a, b, 1, 3, 2);
    theta_(a, b, 2, 4, 3);
    theta_(a, b, 3, 0, 4);
}

fn rho_pi<T>(a: &mut [T], b: &mut [T])
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    let t = a[1];
    b[0] = a[10];
    a[10] = rotate_left(t, 1);
    rho_pi_(a, b, 7, 3);
    rho_pi_(a, b, 11, 6);
    rho_pi_(a, b, 17, 10);
    rho_pi_(a, b, 18, 15);
    rho_pi_(a, b, 3, 21);
    rho_pi_(a, b, 5, 28);
    rho_pi_(a, b, 16, 36);
    rho_pi_(a, b, 8, 45);
    rho_pi_(a, b, 21, 55);
    rho_pi_(a, b, 24, 2);
    rho_pi_(a, b, 4, 14);
    rho_pi_(a, b, 15, 27);
    rho_pi_(a, b, 23, 41);
    rho_pi_(a, b, 19, 56);
    rho_pi_(a, b, 13, 8);
    rho_pi_(a, b, 12, 25);
    rho_pi_(a, b, 2, 43);
    rho_pi_(a, b, 20, 62);
    rho_pi_(a, b, 14, 18);
    rho_pi_(a, b, 22, 39);
    rho_pi_(a, b, 9, 61);
    rho_pi_(a, b, 6, 20);
    rho_pi_(a, b, 1, 44);
}

fn rho_pi_<T>(a: &mut [T], b: &mut [T], m: usize, n: u32)
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    let t = b[0];
    b[0] = a[m];
    a[m] = rotate_left(t, n);
}

fn rotate_left<T>(value: T, shift: u32) -> T
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    (value << shift) | (value >> (64 - shift))
}

fn chi<T>(a: &mut [T])
where
    T: Not<Output = T> + BitAnd<Output = T> + BitXor<Output = T> + Default + Copy,
{
    let mut b = [T::default(); 5];
    chi_(a, &mut b, 0);
    chi_(a, &mut b, 5);
    chi_(a, &mut b, 10);
    chi_(a, &mut b, 15);
    chi_(a, &mut b, 20);
}

fn chi_<T>(a: &mut [T], b: &mut [T], n: usize)
where
    T: Not<Output = T> + BitAnd<Output = T> + BitXor<Output = T> + Copy,
{
    b[0] = a[n];
    b[1] = a[n + 1];
    b[2] = a[n + 2];
    b[3] = a[n + 3];
    b[4] = a[n + 4];
    a[n] = b[0] ^ ((!b[1]) & b[2]);
    a[n + 1] = b[1] ^ ((!b[2]) & b[3]);
    a[n + 2] = b[2] ^ ((!b[3]) & b[4]);
    a[n + 3] = b[3] ^ ((!b[4]) & b[0]);
    a[n + 4] = b[4] ^ ((!b[0]) & b[1]);
}

fn iota<T, U>(a: &mut [T], x: U)
where
    T: BitXorAssign<U> + Copy,
{
    a[0] ^= x;
}

pub const fn normalize_endianess(x: u32) -> u32 {
    x.to_be()
}

pub fn function_selector_to_hex(x: u32) -> String {
    format!("0x{:0width$x}", x, width = std::mem::size_of::<u32>() * 2)
}

// playing around with AVX2
pub fn theta_avx2(a: &mut [u64], b: &mut [u64; 5]) {
    unsafe {
        let b_0_3 = [
            _mm256_set_epi64x(a[0] as i64, a[1] as i64, a[2] as i64, a[3] as i64),
            _mm256_set_epi64x(a[5] as i64, a[6] as i64, a[7] as i64, a[8] as i64),
            _mm256_set_epi64x(a[10] as i64, a[11] as i64, a[12] as i64, a[13] as i64),
            _mm256_set_epi64x(a[15] as i64, a[16] as i64, a[17] as i64, a[18] as i64),
            _mm256_set_epi64x(a[20] as i64, a[21] as i64, a[22] as i64, a[23] as i64),
        ];

        // reduce with _mm256_xor_si256
        let b_0_3 = _mm256_xor_si256(
            _mm256_xor_si256(
                _mm256_xor_si256(b_0_3[0], b_0_3[1]),
                _mm256_xor_si256(b_0_3[2], b_0_3[3]),
            ),
            b_0_3[4],
        );

        b[0] = _mm256_extract_epi64(b_0_3, 3) as u64;
        b[1] = _mm256_extract_epi64(b_0_3, 2) as u64;
        b[2] = _mm256_extract_epi64(b_0_3, 1) as u64;
        b[3] = _mm256_extract_epi64(b_0_3, 0) as u64;
        b[4] = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24];

        theta_(a, b, 4, 1, 0);
        theta_(a, b, 0, 2, 1);
        theta_(a, b, 1, 3, 2);
        theta_(a, b, 2, 4, 3);
        theta_(a, b, 3, 0, 4);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theta_equivalence() {
        let mut a = [0u64];
        let mut b = [0u64; 5];

        // Initialize `a` and `b` with some values
        for i in 0..25 {
            a[i] = i as u64;
        }
        for i in 0..5 {
            b[i] = (i * 5) as u64;
        }

        let mut a_theta = a;
        let mut b_theta = b;
        theta(&mut a_theta, &mut b_theta);

        #[cfg(target_feature = "avx2")]
        {
            let mut a_theta_avx2 = a;
            let mut b_theta_avx2 = b;
            theta_avx2(&mut a_theta_avx2, &mut b_theta_avx2);

            assert_eq!(a_theta, a_theta_avx2);
            assert_eq!(b_theta, b_theta_avx2);
        }

        #[cfg(not(target_feature = "avx2"))]
        {
            println!("AVX2 not enabled, skipping AVX2 test");
        }
    }

    #[test]
    fn test_shift() {
        let v0: u64 = 0xFFFFFFFFFFFFFFFF;
        let v1: u64 = 0xFFFFFFFFFFFFFFFF;
        let v2: u64 = 0xFFFFFFFFFFFFFFFF;
        let v3: u64 = 0xFFFFFFFFFFFFFFFF;

        let v = unsafe { _mm256_set_epi64x(v0 as i64, v1 as i64, v2 as i64, v3 as i64) };

        let v = unsafe { _mm256_sll_epi64(v, _mm_set1_epi64x(1)) };

        // pull out each as u64
        let v0 = unsafe { _mm256_extract_epi64(v, 0) } as u64;
        let v1 = unsafe { _mm256_extract_epi64(v, 1) } as u64;
        let v2 = unsafe { _mm256_extract_epi64(v, 2) } as u64;
        let v3 = unsafe { _mm256_extract_epi64(v, 3) } as u64;

        assert_eq!(v0, 0xFFFFFFFFFFFFFFFE);
        assert_eq!(v1, 0xFFFFFFFFFFFFFFFE);
        assert_eq!(v2, 0xFFFFFFFFFFFFFFFE);
        assert_eq!(v3, 0xFFFFFFFFFFFFFFFE);

        println!("{:?}", v);
    }
}
