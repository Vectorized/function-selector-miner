# Function Selector Miner

Simple and fast Solidity function selector miner (CPU based).

Uses AVX2 instructions and multithreading to compute hashes in parallel.

Should be able to mine most selectors under a minute with a core i7-12700F.

## Requirements

Rust compiler. Tested with rustc 1.72.0.

## Running

```
cargo run --release <function_name> <function_params> <target_selector> [num_threads]
```