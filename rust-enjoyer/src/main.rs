use rust_enjoyer::*;
use std::str;

use rayon::prelude::*;
use std::env;
use std::process;

fn main() {
    #[cfg(target_feature = "avx2")]
    println!("AVX2 enabled");
    #[cfg(not(target_feature = "avx2"))]
    println!("AVX2 disabled");

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
        println!("u64: {}", std::mem::size_of::<u64>());
        process::exit(-1);
    }

    println!("Function name: {}", args[1]);
    println!("Function params: {}", args[2]);
    println!(
        "Target selector: {}",
        function_selector_to_hex(normalize_endianess(selector))
    );

    let num_threads = num_cpus::get();
    let end = 0xfffffffff0000000usize;
    let go = std::sync::atomic::AtomicBool::new(true);

    #[cfg(target_feature = "avx2")]
    const STEP: usize = 1;
    #[cfg(not(target_feature = "avx2"))]
    const STEP: usize = 1;

    println!("Starting mining with {num_threads} threads.");

    (0..num_threads).into_par_iter().for_each(|t| {
        let mut i = 0;
        for nonce in (t * STEP..end).step_by((num_threads * STEP) as usize) {
            if !go.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            // #[cfg(not(target_feature = "avx2"))]
            {
                let mut s0 = Sponge::default();

                let o = unsafe { s0.fill_sponge(&function_name, nonce as u64, &function_params) };
                unsafe { s0.chars[o] = 0x01 };

                let c0 = unsafe { s0.compute_selectors() };
                if c0 == selector {
                    unsafe { s0.fill_sponge(&function_name, nonce as u64, &function_params) };
                    unsafe { s0.chars[o] = 0x00 };

                    let mut out = [0u8; 200];
                    out[..o].copy_from_slice(unsafe { &s0.chars[..o] });
                    let out = unsafe { str::from_utf8_unchecked(&out) };
                    println!("Function found: {}", out);

                    go.store(false, std::sync::atomic::Ordering::Relaxed);
                }
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
