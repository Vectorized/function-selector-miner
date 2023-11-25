#include <iostream>
#include <iomanip>
#include <sstream>
#include <fstream>
#include <string>
#include <cstring>
#include <stdint.h>
#include <omp.h>

inline char *writeDecimal(char *out, uint64_t x)
{
    char buff[64], *end = buff + 32, *p = end;
    do
    {
        *(--p) = (x % 10) + 48;
    } while (x /= 10);
    memcpy(out, p, 32);
    return out + (end - p);
}

inline std::string functionSelectorToHex(uint32_t x)
{
    std::stringstream ss;
    ss << "0x" << std::setfill('0') << std::setw(sizeof(uint32_t) * 2) << std::hex << x;
    return ss.str();
}

#if defined(__AVX2__)
#include <immintrin.h>

struct V
{
    __m256i v;
    inline V() {}
    inline V(const __m256i &v) : v(v) {}
    inline V(uint64_t v0, uint64_t v1, uint64_t v2, uint64_t v3)
    {
        v = _mm256_set_epi64x(v3, v2, v1, v0);
    }
};

inline V operator^(const V &a, const V &b)
{
    return V(_mm256_xor_si256(a.v, b.v));
}

inline V operator^(const V &a, uint64_t b)
{
    return V(_mm256_xor_si256(a.v, _mm256_set1_epi64x(b)));
}

inline V operator|(const V &a, const V &b)
{
    return V(_mm256_or_si256(a.v, b.v));
}

inline V operator<<(const V &a, uint64_t i)
{
    return V(_mm256_sll_epi64(a.v, _mm_set1_epi64x(i)));
}

inline V operator>>(const V &a, uint64_t i)
{
    return V(_mm256_srl_epi64(a.v, _mm_set1_epi64x(i)));
}

inline V operator&(const V &a, const V &b)
{
    return V(_mm256_and_si256(a.v, b.v));
}

inline V operator~(const V &a)
{
    return V(_mm256_xor_si256(a.v, _mm256_set1_epi64x(0xffffffffffffffffull)));
}
#endif

#define ROL(X, S) (((X) << S) | ((X) >> (64 - S)))

#if defined(__AVX512F__) && defined(__AVX512VL__)
#undef ROL
#define ROL(X, S) V(_mm256_rol_epi64((X).v, S))
#endif

#define THETA_(M, N, O)        \
    t = b[M] ^ ROL(b[N], 1);   \
    a[O + 0] = a[O + 0] ^ t;   \
    a[O + 5] = a[O + 5] ^ t;   \
    a[O + 10] = a[O + 10] ^ t; \
    a[O + 15] = a[O + 15] ^ t; \
    a[O + 20] = a[O + 20] ^ t;

#define THETA()                                 \
    b[0] = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20]; \
    b[1] = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21]; \
    b[2] = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22]; \
    b[3] = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23]; \
    b[4] = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24]; \
    THETA_(4, 1, 0);                            \
    THETA_(0, 2, 1);                            \
    THETA_(1, 3, 2);                            \
    THETA_(2, 4, 3);                            \
    THETA_(3, 0, 4);

#define RHO_PI_(M, N) \
    t = b[0];         \
    b[0] = a[M];      \
    a[M] = ROL(t, N);

#define RHO_PI()       \
    t = a[1];          \
    b[0] = a[10];      \
    a[10] = ROL(t, 1); \
    RHO_PI_(7, 3);     \
    RHO_PI_(11, 6);    \
    RHO_PI_(17, 10);   \
    RHO_PI_(18, 15);   \
    RHO_PI_(3, 21);    \
    RHO_PI_(5, 28);    \
    RHO_PI_(16, 36);   \
    RHO_PI_(8, 45);    \
    RHO_PI_(21, 55);   \
    RHO_PI_(24, 2);    \
    RHO_PI_(4, 14);    \
    RHO_PI_(15, 27);   \
    RHO_PI_(23, 41);   \
    RHO_PI_(19, 56);   \
    RHO_PI_(13, 8);    \
    RHO_PI_(12, 25);   \
    RHO_PI_(2, 43);    \
    RHO_PI_(20, 62);   \
    RHO_PI_(14, 18);   \
    RHO_PI_(22, 39);   \
    RHO_PI_(9, 61);    \
    RHO_PI_(6, 20);    \
    RHO_PI_(1, 44);

#define CHI_(N)                         \
    b[0] = a[N + 0];                    \
    b[1] = a[N + 1];                    \
    b[2] = a[N + 2];                    \
    b[3] = a[N + 3];                    \
    b[4] = a[N + 4];                    \
    a[N + 0] = b[0] ^ ((~b[1]) & b[2]); \
    a[N + 1] = b[1] ^ ((~b[2]) & b[3]); \
    a[N + 2] = b[2] ^ ((~b[3]) & b[4]); \
    a[N + 3] = b[3] ^ ((~b[4]) & b[0]); \
    a[N + 4] = b[4] ^ ((~b[0]) & b[1]);

#define CHI() \
    CHI_(0);  \
    CHI_(5);  \
    CHI_(10); \
    CHI_(15); \
    CHI_(20);

#define IOTA(X) a[0] = a[0] ^ X;

#define ITER(X) \
    THETA();    \
    RHO_PI();   \
    CHI();      \
    IOTA(X);

#define ITERS()               \
    ITER(0x0000000000000001); \
    ITER(0x0000000000008082); \
    ITER(0x800000000000808a); \
    ITER(0x8000000080008000); \
    ITER(0x000000000000808b); \
    ITER(0x0000000080000001); \
    ITER(0x8000000080008081); \
    ITER(0x8000000000008009); \
    ITER(0x000000000000008a); \
    ITER(0x0000000000000088); \
    ITER(0x0000000080008009); \
    ITER(0x000000008000000a); \
    ITER(0x000000008000808b); \
    ITER(0x800000000000008b); \
    ITER(0x8000000000008089); \
    ITER(0x8000000000008003); \
    ITER(0x8000000000008002); \
    ITER(0x8000000000000080); \
    ITER(0x000000000000800a); \
    ITER(0x800000008000000a); \
    ITER(0x8000000080008081); \
    ITER(0x8000000000008080); \
    ITER(0x0000000080000001); \
    ITER(0x8000000080008008);

#if defined(__AVX2__)
#define COMPUTE_SELECTORS(C0, C1, C2, C3, SPONGE)    \
    {                                                \
        V *a = SPONGE, b[5], t;                      \
        ITERS()                                      \
        memcpy(&C0, ((uint64_t *)&(a[0].v)) + 0, 4); \
        memcpy(&C1, ((uint64_t *)&(a[0].v)) + 1, 4); \
        memcpy(&C2, ((uint64_t *)&(a[0].v)) + 2, 4); \
        memcpy(&C3, ((uint64_t *)&(a[0].v)) + 3, 4); \
    }
#else
#define COMPUTE_SELECTORS(C, SPONGE)   \
    {                                  \
        uint64_t *a = SPONGE, b[5], t; \
        ITERS()                        \
        memcpy(&C, a, 4);              \
    }
#endif

inline uint32_t normalizeEndianess(uint32_t x)
{
    union
    {
        uint32_t i;
        char c[4];
    } bint = {0x01020304};
    if (bint.c[0] == 1)
        return x;
    x = ((x >> 8) & 0x00FF00FF) | ((x & 0x00FF00FF) << 8);
    return (x >> 16) | (x << 16);
}

struct SmallString
{
    char data[128];
    uint64_t length;

    inline SmallString(const char *s)
    {
        length = strlen(s);
        if (length < 128)
            strcpy(data, s);
    }
};

inline char *fillSponge(char *sponge, const SmallString &s)
{
    memcpy(sponge, s.data, s.length);
    return sponge + s.length;
}

inline char *fillSponge(char *sponge, const SmallString &functionName, uint64_t nonce, const SmallString &functionParams)
{
    char *o = sponge;
    o = fillSponge(o, functionName);
    o = writeDecimal(o, nonce);
    o = fillSponge(o, functionParams);
    const char *end = sponge + 200;
    for (char *c = o; c < end; ++c)
        *c = 0;
    sponge[135] = 0x80u;
    return o;
}

int main(int argc, char *argv[])
{
    if (argc < 4)
    {
        std::cout << "Usage: <function name> <function params> <target selector>\n"
                  << std::flush;
        return -1;
    }

    const uint32_t selector = normalizeEndianess(std::stol(argv[3], nullptr, 16));
    const SmallString functionName(argv[1]);
    const SmallString functionParams(argv[2]);

    if (functionName.length + functionParams.length >= 115)
    {
        std::cout << "Total length of <function name> and <function params> must be under 115 bytes.";
        return -1;
    }

    if (sizeof(char) != 1 || sizeof(uint64_t) != 8)
    {
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

    union Sponge
    {
        uint64_t uint64s[25];
        char chars[200];
    };

#pragma omp parallel for
    for (uint64_t t = 0; t < numThreads; ++t)
    {
#if defined(__AVX2__)
        std::cout << "Using AVX2\n";
#define STEP 4
#else
#define STEP 1
#endif
        uint64_t i = 0;
        for (uint64_t nonce = t * STEP; nonce < end && go; nonce += numThreads * STEP)
        {
#define CHECK_SELECTOR(I)                                                         \
    if (c##I == selector)                                                         \
    {                                                                             \
        *fillSponge(s##I.chars, functionName, nonce + I, functionParams) = 0x00u; \
        std::cout << "Function found: " << s##I.chars << "\n";                    \
        go = false;                                                               \
    }

#if defined(__AVX2__)
            Sponge s0, s1, s2, s3;
            *fillSponge(s0.chars, functionName, nonce + 0, functionParams) = 0x01u;
            *fillSponge(s1.chars, functionName, nonce + 1, functionParams) = 0x01u;
            *fillSponge(s2.chars, functionName, nonce + 2, functionParams) = 0x01u;
            *fillSponge(s3.chars, functionName, nonce + 3, functionParams) = 0x01u;
            V sponge[25];
#define SET_SPONGE_(I) sponge[I] = V(s0.uint64s[I], s1.uint64s[I], s2.uint64s[I], s3.uint64s[I]);
#define SET_SPONGE(I) SET_SPONGE_(I + 0) SET_SPONGE_(I + 1) SET_SPONGE_(I + 2) SET_SPONGE_(I + 3) SET_SPONGE_(I + 4)
            SET_SPONGE(0) SET_SPONGE(5) SET_SPONGE(10) SET_SPONGE(15) SET_SPONGE(20)
            uint32_t c0, c1, c2, c3;
            COMPUTE_SELECTORS(c0, c1, c2, c3, sponge);
            CHECK_SELECTOR(0)
            CHECK_SELECTOR(1)
            CHECK_SELECTOR(2)
            CHECK_SELECTOR(3)
#else
            Sponge s0;
            *fillSponge(s0.chars, functionName, nonce, functionParams) = 0x01u;
            uint32_t c0;
            COMPUTE_SELECTORS(c0, s0.uint64s);
            CHECK_SELECTOR(0)
#endif
            if (t == 0) if ((++i & 0x3fffff) == 0) std::cout << nonce << " hashes done.\n";
        }
    }
    return 0;
}
