#include <iostream>
#include <iomanip>
#include <sstream>
#include <fstream>
#include <string>
#include <cstring>
#include <stdint.h>
#include <omp.h>

inline char* writeDecimal(char* out, uint64_t x)
{
    char buff[32], *end = buff + 32, *p = end;
    do { *(--p) = (x % 10) + 48; } while (x /= 10);
    const uint64_t n = end - p;
    for (uint64_t i = 0; i < n; ++i) out[i] = p[i];
    return out + n;
}

inline std::string functionSelectorToHex(uint32_t x)
{
    std::stringstream ss;
    ss << "0x" << std::setfill('0') << std::setw(sizeof(uint32_t) * 2) << std::hex << x;
    return ss.str();
}

#define ROL(X, S) (((X) << S) | ((X) >> (64u - S)))

#define ROL1(X) ROL(X, 1u)

#define THETA_(M, N, O) t = b[M] ^ ROL1(b[N]); a[O + 0] ^= t; a[O + 5] ^= t; a[O + 10] ^= t; a[O + 15] ^= t; a[O + 20] ^= t;

#define THETA() \
b[0] = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20]; \
b[1] = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21]; \
b[2] = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22]; \
b[3] = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23]; \
b[4] = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24]; \
THETA_(4, 1, 0); THETA_(0, 2, 1); THETA_(1, 3, 2); THETA_(2, 4, 3); THETA_(3, 0, 4);

#define RHO_PI_(M, N) t = b[0]; b[0] = a[M]; a[M] = ROL(t, N); \

#define RHO_PI() t = a[1]; b[0] = a[10]; a[10] = ROL1(t); \
RHO_PI_(7, 3); RHO_PI_(11, 6); RHO_PI_(17, 10); RHO_PI_(18, 15); RHO_PI_(3, 21); RHO_PI_(5, 28); \
RHO_PI_(16, 36); RHO_PI_(8, 45); RHO_PI_(21, 55); RHO_PI_(24, 2); RHO_PI_(4, 14); RHO_PI_(15, 27); \
RHO_PI_(23, 41); RHO_PI_(19, 56); RHO_PI_(13, 8); RHO_PI_(12, 25); RHO_PI_(2, 43); RHO_PI_(20, 62); \
RHO_PI_(14, 18); RHO_PI_(22, 39); RHO_PI_(9, 61); RHO_PI_(6, 20); RHO_PI_(1, 44);

#define CHI_(N) \
b[0] = a[N + 0]; b[1] = a[N + 1]; b[2] = a[N + 2]; b[3] = a[N + 3]; b[4] = a[N + 4]; \
a[N + 0] = b[0] ^ ((~b[1]) & b[2]); \
a[N + 1] = b[1] ^ ((~b[2]) & b[3]); \
a[N + 2] = b[2] ^ ((~b[3]) & b[4]); \
a[N + 3] = b[3] ^ ((~b[4]) & b[0]); \
a[N + 4] = b[4] ^ ((~b[0]) & b[1]);

#define CHI() CHI_(0); CHI_(5); CHI_(10); CHI_(15); CHI_(20);

#define IOTA(X) a[0] ^= X;

#define ITER(X) THETA(); RHO_PI(); CHI(); IOTA(X);

inline uint32_t functionSelector(uint64_t *a)
{
    uint64_t b[5], t;

    ITER(0x0000000000000001);
    ITER(0x0000000000008082);
    ITER(0x800000000000808a);
    ITER(0x8000000080008000);
    ITER(0x000000000000808b);
    ITER(0x0000000080000001);
    ITER(0x8000000080008081);
    ITER(0x8000000000008009);
    ITER(0x000000000000008a);
    ITER(0x0000000000000088);
    ITER(0x0000000080008009);
    ITER(0x000000008000000a);
    ITER(0x000000008000808b);
    ITER(0x800000000000008b);
    ITER(0x8000000000008089);
    ITER(0x8000000000008003);
    ITER(0x8000000000008002);
    ITER(0x8000000000000080);
    ITER(0x000000000000800a);
    ITER(0x800000008000000a);
    ITER(0x8000000080008081);
    ITER(0x8000000000008080);
    ITER(0x0000000080000001);
    ITER(0x8000000080008008);

    uint32_t result;
    memcpy(&result, a, 4);
    return result;
}

inline uint32_t normalizeEndianess(uint32_t x)
{
    union {
        uint32_t i;
        char c[4];
    } bint = {0x01020304};
    if (bint.c[0] == 1) return x;
    x = ((x >> 8) & 0x00FF00FF) | ((x & 0x00FF00FF) << 8);
    return (x >> 16) | (x << 16);    
}

inline char * fillSponge(char *sponge, const std::string &s)
{
    uint64_t n = s.size();
    for (uint64_t i = 0; i < n; ++i) sponge[i] = s[i];
    return sponge + n;
}

inline char * fillSponge(char *sponge, const std::string &functionName, uint64_t nonce, const std::string &functionParams)
{
    char *o = sponge;
    o = fillSponge(o, functionName);
    o = writeDecimal(o, nonce);
    o = fillSponge(o, functionParams);
    const char *end = sponge + 200;
    for (char *c = o; c < end; ++c) *c = 0;
    sponge[135] = 0x80u;
    return o;
}

int main(int argc, char * argv[])
{
    if (argc < 4) {
        std::cout << "Usage: <function name> <function params> <target selector>\n" << std::flush;
        return -1;
    }

    const uint32_t selector = normalizeEndianess(std::stol(argv[3], nullptr, 16));
    const std::string functionName(argv[1]);
    const std::string functionParams(argv[2]);

    if (functionName.size() + functionParams.size() >= 115) {
        std::cout << "Total length of <function name> and <function params> must be under 115 bytes.";
        return -1;
    }

    if (sizeof(char) != 1 || sizeof(uint64_t) != 8) {
        std::cout << "Incompatible architecture\n";
        return -1;
    }

    std::cout << "Function name: " << argv[1] << "\n";
    std::cout << "Function params: " << argv[2] << "\n";
    std::cout << "Target selector: " << functionSelectorToHex(normalizeEndianess(selector)) << "\n";
    
    bool go = true;

    const uint64_t numThreads = omp_get_max_threads();
    const uint64_t end = 0xfffffffff0000000ull;

    std::cout << "Starting mining with " << numThreads << " threads...\n";

#pragma omp parallel for
    for (uint64_t t = 0; t < numThreads; ++t) {
        uint64_t i = 0;
        for (uint64_t nonce = t; nonce < end && go; nonce += numThreads) {
            union {
                uint64_t uint64s[25];
                char chars[200];
            } sponge;
            *fillSponge(sponge.chars, functionName, nonce, functionParams) = 0x01u;
            uint32_t computed = functionSelector(sponge.uint64s);
            if (computed == selector) {
                *fillSponge(sponge.chars, functionName, nonce, functionParams) = 0x00u;
                std::cout << "Function found: " << sponge.chars << "\n";
                go = false;
            }
            if (t == 0) if ((++i & 0x3fffff) == 0) std::cout << nonce << " hashes done.\n";
        }
    }
    return 0;
}
