#[cfg(target_feature = "avx2")]
use function_selector_miner::sponges_avx::SpongesAvx;
use rayon::ThreadPoolBuilder;

use function_selector_miner::*;

use rayon::prelude::*;
use std::env;
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Instant;

macro_rules! log_progress {
    ($thread_idx:expr, $idx:expr, $mask:expr, $nonce:expr) => {
        if $thread_idx == 0 {
            $idx += 1;
            if $idx & $mask == 0 {
                println!("{} hashes done.", $nonce);
            }
        }
    };
}

macro_rules! log_result_and_break {
    ($out:expr, $stopwatch:expr, $go:expr) => {
        println!("Function found: {} in {:.02?}", $out, $stopwatch.elapsed());
        $go.store(false, Ordering::Relaxed);
    };
}

fn mine<const N: usize>(
    function_name: &SmallString,
    function_params: &SmallString,
    selector: u32,
    num_threads: usize,
) {
    let end = 0xfffffffff0000000u64;
    let go = AtomicBool::new(true);

    #[cfg(target_feature = "avx2")]
    const STEP: u64 = 4;
    #[cfg(not(target_feature = "avx2"))]
    const STEP: u64 = 1;

    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Failed to create thread pool.");

    println!("Starting mining with {num_threads} threads.");

    let stopwatch = Instant::now();

    (0..num_threads).into_par_iter().for_each(|thread_idx| {
        let mut idx = 0u64;
        let mut nonce = thread_idx as u64 * STEP;
        let step = num_threads as u64 * STEP;
        while nonce < end {
            nonce += step;
            if !go.load(Ordering::Relaxed) {
                break;
            }

            #[cfg(not(target_feature = "avx2"))]
            {
                log_progress!(thread_idx, idx, 0x3fffff, nonce);

                let mut s0 = Sponge::default();
                unsafe { s0.fill::<N>(&function_name, nonce, &function_params) };

                if selector == unsafe { s0.compute_selectors() } {
                    let out = unsafe {
                        s0.fill_and_get_name::<N>(&function_name, nonce, &function_params)
                    };
                    log_result_and_break!(out, stopwatch, go);
                }
            }
            #[cfg(target_feature = "avx2")]
            {
                log_progress!(thread_idx, idx, 0x1fffff, nonce);

                let mut sponges =
                    unsafe { SpongesAvx::new::<N>(&function_name, nonce, &function_params) };

                let maybe_idx = unsafe {
                    sponges
                        .compute_selectors()
                        .iter()
                        .position(|&x| x == selector)
                };

                let Some(found_idx) = maybe_idx else {
                    continue;
                };

                // Match found.
                let out = unsafe {
                    Sponge::default().fill_and_get_name::<N>(
                        &function_name,
                        nonce + found_idx as u64,
                        &function_params,
                    )
                };
                log_result_and_break!(out, stopwatch, go);
            }
        }
    });
}

fn main() {
    #[cfg(target_feature = "avx2")]
    println!("AVX2 enabled.");
    #[cfg(not(target_feature = "avx2"))]
    println!("AVX2 disabled.");

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Usage: <function name> <function params> <target selector> [num_threads]");
        process::exit(-1);
    }

    // Remove any leading 0x.
    let selector = args[3].to_lowercase();
    let selector = selector.trim_start_matches("0x");
    let selector = u32::from_str_radix(selector, 16)
        .expect("Invalid number.")
        .to_be();
    let function_name = SmallString::new(&args[1]);
    let function_params = SmallString::new(&args[2]);

    if function_name.length + function_params.length >= 115 {
        println!("Total length of <function name> and <function params> must be under 115 bytes.");
        process::exit(-1);
    }

    if std::mem::size_of::<u64>() != 8 {
        println!("Incompatible architecture.");
        println!("u64: {}", std::mem::size_of::<u64>());
        process::exit(-1);
    }

    println!("Function name: {}", args[1]);
    println!("Function params: {}", args[2]);
    println!(
        "Target selector: 0x{}",
        &(&format!("{:x}", (selector.to_be() as u64) | 0x0100000000))[1..]
    );

    let num_threads = args
        .get(4)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| num_cpus::get());

    if function_name.length <= 64 && function_params.length <= 64 {
        mine::<64>(&function_name, &function_params, selector, num_threads);
    } else {
        mine::<0>(&function_name, &function_params, selector, num_threads);
    }
}
