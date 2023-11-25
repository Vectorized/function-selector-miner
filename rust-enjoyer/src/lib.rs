pub mod small_string;
pub mod sponge;
#[cfg(target_feature = "avx2")]
pub mod sponges_avx;

pub use small_string::*;
pub use sponge::*;

use std::ops::{BitAnd, BitOr, BitXor, BitXorAssign, Not, Shl, Shr};

fn iters<T>(a: &mut [T; 25])
where
    T: BitXorAssign
        + BitXorAssign<u64>
        + BitXor<Output = T>
        + BitOr<Output = T>
        + BitAnd<Output = T>
        + Not<Output = T>
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + Default
        + Copy,
{
    let b = &mut <[T; 5]>::default();
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
        iter(a, b, v);
    });
}

fn iter<T>(a: &mut [T; 25], b: &mut [T; 5], x: u64)
where
    T: BitXorAssign
        + BitXorAssign<u64>
        + BitXor<Output = T>
        + BitOr<Output = T>
        + BitAnd<Output = T>
        + Not<Output = T>
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + Default
        + Copy,
{
    theta(a, b);
    rho_pi(a, b);
    chi(a);
    iota(a, x);
}

pub fn theta<T>(a: &mut [T; 25], b: &mut [T; 5])
where
    T: BitXorAssign
        + BitXor<Output = T>
        + Shl<u32, Output = T>
        + Shr<u32, Output = T>
        + BitOr<Output = T>
        + Copy,
{
    [
        (0, 5, 10, 15, 20), // i, j, k, l, m
        (1, 6, 11, 16, 21),
        (2, 7, 12, 17, 22),
        (3, 8, 13, 18, 23),
        (4, 9, 14, 19, 24),
    ]
    .into_iter()
    .for_each(|(i, j, k, l, m)| {
        b[i] = a[i] ^ a[j] ^ a[k] ^ a[l] ^ a[m];
    });

    [
        (4, 1), // m, n
        (0, 2),
        (1, 3),
        (2, 4),
        (3, 0),
    ]
    .into_iter()
    .enumerate()
    .for_each(|(i, (m, n))| {
        let t = b[m] ^ rotate_left(b[n], 1);
        a[i] ^= t;
        a[i + 5] ^= t;
        a[i + 10] ^= t;
        a[i + 15] ^= t;
        a[i + 20] ^= t;
    });
}

fn rho_pi<T>(a: &mut [T; 25], b: &mut [T; 5])
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    let t = a[1];
    b[0] = a[10];
    a[10] = rotate_left(t, 1);

    [
        (7, 3), // m, n
        (11, 6),
        (17, 10),
        (18, 15),
        (3, 21),
        (5, 28),
        (16, 36),
        (8, 45),
        (21, 55),
        (24, 2),
        (4, 14),
        (15, 27),
        (23, 41),
        (19, 56),
        (13, 8),
        (12, 25),
        (2, 43),
        (20, 62),
        (14, 18),
        (22, 39),
        (9, 61),
        (6, 20),
        (1, 44),
    ]
    .into_iter()
    .for_each(|(m, n)| {
        let t = b[0];
        b[0] = a[m];
        a[m] = rotate_left(t, n);
    });
}

fn rotate_left<T>(value: T, shift: u32) -> T
where
    T: Copy + Shl<u32, Output = T> + Shr<u32, Output = T> + BitOr<Output = T>,
{
    (value << shift) | (value >> (64 - shift))
}

fn chi<T>(a: &mut [T; 25])
where
    T: Not<Output = T> + BitAnd<Output = T> + BitXor<Output = T> + Default + Copy,
{
    let mut b = <[T; 5]>::default();
    [0, 5, 10, 15, 20].into_iter().for_each(|n| {
        // b[0..5] = a[n..n+5]
        b.iter_mut()
            .enumerate()
            .for_each(|(idx, b_i)| *b_i = a[n + idx]);

        a[n] = b[0] ^ ((!b[1]) & b[2]);
        a[n + 1] = b[1] ^ ((!b[2]) & b[3]);
        a[n + 2] = b[2] ^ ((!b[3]) & b[4]);
        a[n + 3] = b[3] ^ ((!b[4]) & b[0]);
        a[n + 4] = b[4] ^ ((!b[0]) & b[1]);
    });
}

fn iota<T, U>(a: &mut [T], x: U)
where
    T: BitXorAssign<U> + Copy,
{
    a[0] ^= x;
}
