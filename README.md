# Function Selector Miner

Simple and fast Solidity function selector miner (CPU based).

Uses AVX2 instructions and multithreading to compute hashes in parallel.

Should be able to mine any selector under a minute with a core i7-12700F.

## Requirements

C++ compiler with OpenMP support.

## Building

```
g++ main.cpp -O3 -march=native -DNDEBUG -fopenmp -o function_selector_miner
```

## Running

```
./function_selector_miner <function name> <function params> <target selector>
```

For example: 

```
./function_selector_miner "someFunction" "(uint256,address)" "0x12345678"
```
