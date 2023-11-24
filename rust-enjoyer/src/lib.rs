use std::arch::x86_64::*;

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

pub fn theta_avx2(a: &mut [u64; 25], b: &mut [u64; 5]) {
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

const ROL: fn(u64, u32) -> u64 = |x, s| x.rotate_left(s);

pub fn theta(a: &mut [u64; 25], b: &mut [u64; 5]) {
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

fn theta_(a: &mut [u64; 25], b: &[u64; 5], m: usize, n: usize, o: usize) {
    let t = b[m] ^ ROL(b[n], 1);
    a[o + 0] ^= t;
    a[o + 5] ^= t;
    a[o + 10] ^= t;
    a[o + 15] ^= t;
    a[o + 20] ^= t;
}

fn rho_pi(a: &mut [u64; 25], b: &mut [u64; 5]) {
    let t = a[1];
    b[0] = a[10];
    a[10] = ROL(t, 1);
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

fn rho_pi_(a: &mut [u64], b: &mut [u64], m: usize, n: u32) {
    let t = b[0];
    b[0] = a[m];
    a[m] = ROL(t, n);
}

fn chi(a: &mut [u64]) {
    let mut b = [0u64; 5];
    chi_(a, &mut b, 0);
    chi_(a, &mut b, 5);
    chi_(a, &mut b, 10);
    chi_(a, &mut b, 15);
    chi_(a, &mut b, 20);
}

fn chi_(a: &mut [u64], b: &mut [u64], n: usize) {
    b[0] = a[n + 0];
    b[1] = a[n + 1];
    b[2] = a[n + 2];
    b[3] = a[n + 3];
    b[4] = a[n + 4];
    a[n + 0] = b[0] ^ ((!b[1]) & b[2]);
    a[n + 1] = b[1] ^ ((!b[2]) & b[3]);
    a[n + 2] = b[2] ^ ((!b[3]) & b[4]);
    a[n + 3] = b[3] ^ ((!b[4]) & b[0]);
    a[n + 4] = b[4] ^ ((!b[0]) & b[1]);
}

fn iota(a: &mut [u64], x: u64) {
    a[0] ^= x;
}

pub fn iter(a: &mut [u64; 25], b: &mut [u64; 5], x: u64) {
    // theta_avx2(a, b);
    theta(a, b);
    rho_pi(a, b);
    chi(a);
    iota(a, x);
}

pub fn iters(a: &mut [u64; 25], b: &mut [u64; 5]) {
    iter(a, b, 0x0000000000000001);
    iter(a, b, 0x0000000000008082);
    iter(a, b, 0x800000000000808a);
    iter(a, b, 0x8000000080008000);
    iter(a, b, 0x000000000000808b);
    iter(a, b, 0x0000000080000001);
    iter(a, b, 0x8000000080008081);
    iter(a, b, 0x8000000000008009);
    iter(a, b, 0x000000000000008a);
    iter(a, b, 0x0000000000000088);
    iter(a, b, 0x0000000080008009);
    iter(a, b, 0x000000008000000a);
    iter(a, b, 0x000000008000808b);
    iter(a, b, 0x800000000000008b);
    iter(a, b, 0x8000000000008089);
    iter(a, b, 0x8000000000008003);
    iter(a, b, 0x8000000000008002);
    iter(a, b, 0x8000000000000080);
    iter(a, b, 0x000000000000800a);
    iter(a, b, 0x800000008000000a);
    iter(a, b, 0x8000000080008081);
    iter(a, b, 0x8000000000008080);
    iter(a, b, 0x0000000080000001);
    iter(a, b, 0x8000000080008008);
}

pub const fn normalize_endianess(x: u32) -> u32 {
    x.to_be()
}

pub struct SmallString {
    pub data: [u8; 128],
    pub length: usize,
}

impl SmallString {
    pub fn new(s: &str) -> SmallString {
        let bytes = s.as_bytes();
        let mut data = [0u8; 128];
        data[..bytes.len()].copy_from_slice(bytes);
        SmallString {
            data,
            length: bytes.len(),
        }
    }
}

pub fn fill_sponge_single(sponge: &mut [u8], s: &SmallString) -> usize {
    sponge[..s.length].copy_from_slice(&s.data[..s.length]);
    s.length
}

pub fn fill_sponge(
    sponge: &mut [u8],
    function_name: &SmallString,
    nonce: u64,
    function_params: &SmallString,
) -> usize {
    let mut o = fill_sponge_single(sponge, function_name);
    o += write_decimal(&mut sponge[o..], nonce);
    o += fill_sponge_single(&mut sponge[o..], function_params);

    let end = 200;
    sponge[o..end].fill(0);
    sponge[135] = 0x80;

    o
}

pub fn compute_selectors(sponge: &mut [u64; 25]) -> u32 {
    let mut b = [0u64; 5];

    iters(sponge, &mut b);

    sponge[0] as u32
}

pub fn function_selector_to_hex(x: u32) -> String {
    format!("0x{:0width$x}", x, width = std::mem::size_of::<u32>() * 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theta_equivalence() {
        let mut a = [0u64; 25];
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
}
