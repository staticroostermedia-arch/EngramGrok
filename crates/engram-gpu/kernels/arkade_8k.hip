// Phase 90.D (Suture): THE ARKADE 8K PROJECTOR — Gaussian CSRP
//
// Projects 8192-D Complex32 vectors from the unit hypersphere into R^3
// using a Gaussian Compressed Sensing Random Projection (CSRP) matrix.
// Ported to AMD ROCm/HIP.

#include <hip/hip_runtime.h>
#include <hip/hip_complex.h>
#include <math.h>
#include <stdint.h>

#define DIM        8192
#define K_DIM      3
#define WAVE_SIZE  64 // AMD Wavefronts are usually 64
// "ENGRAM" × epoch 2026
#define GENESIS_ROOT_HASH 0x454E4752414D2026ULL
// JL scale: 1/√3 ≈ 0.5773502691896258
#define JL_SCALE   0.5773502691896258f

__device__ __forceinline__ uint64_t xorshift64(uint64_t s) {
    s ^= s << 13; s ^= s >> 7; s ^= s << 17; return s;
}

__device__ __forceinline__ float uniform01(uint64_t s) {
    // scale by 1.0 / 2^53
    return __saturatef((float)(s >> 11) * (1.0f / (float)(1ULL << 53)));
}

__device__ __forceinline__ float gaussian_sample(uint64_t s1, uint64_t s2) {
    float u1 = uniform01(s1);
    float u2 = uniform01(s2);
    if (u1 < 1e-7f) u1 = 1e-7f;
    return sqrtf(-2.0f * logf(u1)) * cosf(2.0f * 3.14159265358979f * u2);
}

// ── Primary projection kernel ─────────────────────────────────────────────────
extern "C" __global__ void
engram_project_8k_to_3d(const hipFloatComplex *__restrict__ tensor,
                         float3 *__restrict__ point_out,
                         unsigned long long seed) {
    int tid = hipThreadIdx_x;
    float lx = 0.0f, ly = 0.0f, lz = 0.0f;

    for (int i = tid; i < DIM; i += hipBlockDim_x) {
        float mag = hipCabsf(tensor[i]);
        uint64_t base = seed ^ ((uint64_t)(i + 1) * 0x9e3779b97f4a7c15ULL);

        uint64_t sx1 = xorshift64(base ^ 0xAABBCCDD00112233ULL), sx2 = xorshift64(sx1);
        uint64_t sy1 = xorshift64(base ^ 0x11223344AABBCCDDULL), sy2 = xorshift64(sy1);
        uint64_t sz1 = xorshift64(base ^ 0xDEADBEEFCAFEBABEULL), sz2 = xorshift64(sz1);

        lx += mag * gaussian_sample(sx1, sx2) * JL_SCALE;
        ly += mag * gaussian_sample(sy1, sy2) * JL_SCALE;
        lz += mag * gaussian_sample(sz1, sz2) * JL_SCALE;
    }

    // Wavefront reduction
    #pragma unroll
    for (int off = WAVE_SIZE / 2; off > 0; off /= 2) {
        lx += __shfl_down(lx, off, WAVE_SIZE);
        ly += __shfl_down(ly, off, WAVE_SIZE);
        lz += __shfl_down(lz, off, WAVE_SIZE);
    }

    __shared__ float smx[8], smy[8], smz[8];
    int waveId = tid / WAVE_SIZE, laneId = tid % WAVE_SIZE;
    if (laneId == 0) { smx[waveId] = lx; smy[waveId] = ly; smz[waveId] = lz; }
    __syncthreads();

    if (waveId == 0) {
        int nw = hipBlockDim_x / WAVE_SIZE;
        if (nw == 0) nw = 1;

        lx = (laneId < nw) ? smx[laneId] : 0.0f;
        ly = (laneId < nw) ? smy[laneId] : 0.0f;
        lz = (laneId < nw) ? smz[laneId] : 0.0f;
        #pragma unroll
        for (int off = WAVE_SIZE / 2; off > 0; off /= 2) {
            lx += __shfl_down(lx, off, WAVE_SIZE);
            ly += __shfl_down(ly, off, WAVE_SIZE);
            lz += __shfl_down(lz, off, WAVE_SIZE);
        }
        if (tid == 0) {
            point_out[hipBlockIdx_x].x = lx;
            point_out[hipBlockIdx_x].y = ly;
            point_out[hipBlockIdx_x].z = lz;
        }
    }
}

// ── Batch cosine similarity kernel ────────────────────────────────────────────
extern "C" __global__ void
engram_cosine_batch(const hipFloatComplex *__restrict__ query,
                    const hipFloatComplex *__restrict__ candidates,
                    float *__restrict__ scores,
                    int N) {
    if (hipBlockIdx_x >= N) return;
    const hipFloatComplex *cand = candidates + (hipBlockIdx_x * DIM);

    __shared__ float s_dot[8], s_na[8], s_nb[8];
    float ldot = 0.0f, lna = 0.0f, lnb = 0.0f;
    int tid = hipThreadIdx_x;

    for (int i = tid; i < DIM; i += hipBlockDim_x) {
        hipFloatComplex q = query[i], c = cand[i];
        ldot += q.x * c.x + q.y * c.y;
        lna  += q.x * q.x + q.y * q.y;
        lnb  += c.x * c.x + c.y * c.y;
    }

    #pragma unroll
    for (int off = WAVE_SIZE / 2; off > 0; off /= 2) {
        ldot += __shfl_down(ldot, off, WAVE_SIZE);
        lna  += __shfl_down(lna,  off, WAVE_SIZE);
        lnb  += __shfl_down(lnb,  off, WAVE_SIZE);
    }

    int waveId = tid / WAVE_SIZE, laneId = tid % WAVE_SIZE;
    if (laneId == 0) { s_dot[waveId] = ldot; s_na[waveId] = lna; s_nb[waveId] = lnb; }
    __syncthreads();

    if (waveId == 0) {
        int nw = hipBlockDim_x / WAVE_SIZE;
        if (nw == 0) nw = 1;

        ldot = (laneId < nw) ? s_dot[laneId] : 0.0f;
        lna  = (laneId < nw) ? s_na[laneId]  : 0.0f;
        lnb  = (laneId < nw) ? s_nb[laneId]  : 0.0f;
        #pragma unroll
        for (int off = WAVE_SIZE / 2; off > 0; off /= 2) {
            ldot += __shfl_down(ldot, off, WAVE_SIZE);
            lna  += __shfl_down(lna,  off, WAVE_SIZE);
            lnb  += __shfl_down(lnb,  off, WAVE_SIZE);
        }
        if (tid == 0) {
            float denom = sqrtf(lna) * sqrtf(lnb);
            scores[hipBlockIdx_x] = (denom > 1e-8f) ? (ldot / denom) : 0.0f;
        }
    }
}
