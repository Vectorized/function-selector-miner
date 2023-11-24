use std::convert::TryInto;
use std::fs;
use std::io::{self, Write};
use std::str;

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

fn function_selector_to_hex(x: u32) -> String {
    format!("0x{:0width$x}", x, width = std::mem::size_of::<u32>() * 2)
}

#[cfg(target_feature = "avx2")]
use std::arch::x86_64::*;

#[cfg(target_feature = "avx2")]
struct V {
    v: __m256i,
}

#[cfg(target_feature = "avx2")]
impl V {
    fn new(v0: u64, v1: u64, v2: u64, v3: u64) -> V {
        V {
            v: unsafe {
                _mm256_set_epi64x(
                    v3.try_into().unwrap(),
                    v2.try_into().unwrap(),
                    v1.try_into().unwrap(),
                    v0.try_into().unwrap(),
                )
            },
        }
    }
}

const ROL: fn(u64, u32) -> u64 = |x, s| x.rotate_left(s);

fn theta_(a: &mut [u64], b: &[u64], m: usize, n: usize, o: usize) {
    let t = b[m] ^ ROL(b[n], 1);
    a[o + 0] ^= t;
    a[o + 5] ^= t;
    a[o + 10] ^= t;
    a[o + 15] ^= t;
    a[o + 20] ^= t;
}

fn theta(a: &mut [u64], b: &mut [u64]) {
    assert!(a.len() >= 25, "Array 'a' must have at least 25 elements");
    assert!(b.len() >= 5, "Array 'b' must have at least 5 elements");

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

fn rho_pi(a: &mut [u64], b: &mut [u64]) {
    let mut t = a[1];
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

fn iter(a: &mut [u64], b: &mut [u64], x: u64) {
    theta(a, b);
    rho_pi(a, b);
    chi(a);
    iota(a, x);
}

fn iters(a: &mut [u64], b: &mut [u64]) {
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

const fn normalize_endianess(x: u32) -> u32 {
    x.to_be()
}

struct SmallString {
    data: [u8; 128],
    length: usize,
}

impl SmallString {
    fn new(s: &str) -> SmallString {
        let bytes = s.as_bytes();
        let mut data = [0u8; 128];
        data[..bytes.len()].copy_from_slice(bytes);
        SmallString {
            data,
            length: bytes.len(),
        }
    }
}

fn fill_sponge_single(sponge: &mut [u8], s: &SmallString) -> usize {
    sponge[..s.length].copy_from_slice(&s.data[..s.length]);
    s.length
}

fn fill_sponge(
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

fn compute_selectors(sponge: &mut [u64]) -> u32 {
    let mut b = [0u64; 5];
    let mut t = 0u64;

    iters(sponge, &mut b);

    sponge[0] as u32
}

use rayon::prelude::*;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Usage: <function name> <function params> <target selector>");
        process::exit(-1);
    }

    let selector = normalize_endianess(u32::from_str_radix(&args[3], 16).expect("Invalid number"));
    let function_name = SmallString::new(&args[1]);
    let function_params = SmallString::new(&args[2]);

    if function_name.length + function_params.length >= 115 {
        println!("Total length of <function name> and <function params> must be under 115 bytes.");
        process::exit(-1);
    }

    if std::mem::size_of::<u64>() != 8 {
        println!("Incompatible architecture");
        println!("char: {}", std::mem::size_of::<char>());
        println!("u64: {}", std::mem::size_of::<u64>());
        process::exit(-1);
    }

    println!("Function name: {}", args[1]);
    println!("Function params: {}", args[2]);
    println!(
        "Target selector: {}",
        function_selector_to_hex(normalize_endianess(selector))
    );

    union Sponge {
        uint64s: [u64; 25],
        chars: [u8; 200],
    }

    let num_threads = num_cpus::get();
    let end = 0xfffffffff0000000usize;
    let go = std::sync::atomic::AtomicBool::new(true);

    const STEP: usize = 1;

    println!("Starting mining with {num_threads} threads.");

    (0..num_threads).into_par_iter().for_each(|t| {
        let mut i = 0;
        for nonce in (t * STEP..end).step_by((num_threads * STEP) as usize) {
            if !go.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            let mut s0 = Sponge {
                uint64s: [0u64; 25],
            };

            let o = fill_sponge(
                unsafe { &mut s0.chars },
                &function_name,
                nonce as u64,
                &function_params,
            );
            unsafe { s0.chars[o] = 0x01 };

            let c0 = compute_selectors(unsafe { &mut s0.uint64s });
            if c0 == selector {
                fill_sponge(
                    unsafe { &mut s0.chars },
                    &function_name,
                    nonce as u64,
                    &function_params,
                );
                unsafe { s0.chars[o] = 0x00 };

                let mut out = [0u8; 200];
                out[..o].copy_from_slice(unsafe { &s0.chars[..o] });
                let out = unsafe { str::from_utf8_unchecked(&out) };
                println!("Function found: {}", out);

                go.store(false, std::sync::atomic::Ordering::Relaxed);
            }

            // Progress logging for thread 0
            if t == 0 {
                i += 1;
                if i & 0x1fffff == 0 {
                    println!("{nonce:?} hashes done.");
                }
            }
        }
    });
}
