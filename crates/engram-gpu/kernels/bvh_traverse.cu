// LBVH Slab Traversal Kernel — O(log N) K-NN filter stage
//
// Linear BVH (LBVH) node structure mirrors the Rust LBVHNode layout exactly.
// Each node is 32 bytes: min[4] + max[4] floats, where:
//   min[3] = left_child index (as float bits, -1 = leaf)
//   max[3] = right_child index (as float bits)
//   skip_idx = rope pointer (next subtree to visit on miss)
//
// The traversal uses the slab method for ray-AABB intersection testing.
// One thread per query ray. Writes up to max_hits file_offset IDs to hits_out.
//
// U.S. Patent Application No. 19/372,256 — Static Rooster Media

#include <stdint.h>
#include <vector_types.h>

// ── Shared types (must match Rust LBVHNode repr(C)) ─────────────────────────

struct LBVHNode {
    float min[4]; // min[0..2] = AABB min XYZ, min[3] = left_child (as f32 bits) or -1 for leaf
    float max[4]; // max[0..2] = AABB max XYZ, max[3] = right_child (as f32 bits) or file_offset_id
    int   skip_idx; // Rope: next node to visit on AABB miss
    int   _pad[3];
};

struct Ray {
    float origin[3];
    float direction[3];
};

// ── Slab method ray–AABB intersection ────────────────────────────────────────
__device__ __forceinline__ bool ray_aabb_intersect(const Ray &r, const LBVHNode &node) {
    float tmin = -1e30f, tmax = 1e30f;
    #pragma unroll
    for (int axis = 0; axis < 3; ++axis) {
        float inv = (fabsf(r.direction[axis]) > 1e-8f)
                    ? 1.0f / r.direction[axis]
                    : 1e30f;
        float t0 = (node.min[axis] - r.origin[axis]) * inv;
        float t1 = (node.max[axis] - r.origin[axis]) * inv;
        if (t0 > t1) { float tmp = t0; t0 = t1; t1 = tmp; }
        tmin = fmaxf(tmin, t0);
        tmax = fminf(tmax, t1);
    }
    // Also accept if origin is INSIDE the AABB (semantic containment query)
    return (tmin <= tmax) ||
           (r.origin[0] >= node.min[0] && r.origin[0] <= node.max[0] &&
            r.origin[1] >= node.min[1] && r.origin[1] <= node.max[1] &&
            r.origin[2] >= node.min[2] && r.origin[2] <= node.max[2]);
}

// ── LBVH traversal kernel ─────────────────────────────────────────────────────
// One thread per query. Iterative rope traversal — no recursion, no stack overflow.
extern "C" __global__ void
engram_bvh_traverse(const LBVHNode *__restrict__ nodes,
                    const Ray       *__restrict__ rays,
                    uint64_t        *__restrict__ hits_out,
                    int             *__restrict__ hit_counts,
                    int              num_rays,
                    int              num_nodes,
                    int              max_hits) {
    int tid = blockIdx.x * blockDim.x + threadIdx.x;
    if (tid >= num_rays) return;

    const Ray &ray  = rays[tid];
    uint64_t  *hits = hits_out + tid * max_hits;
    int        nhit = 0;

    int node_idx = 0;
    while (node_idx >= 0 && node_idx < num_nodes && nhit < max_hits) {
        const LBVHNode &node = nodes[node_idx];

        if (!ray_aabb_intersect(ray, node)) {
            // Miss — jump via rope to next subtree
            node_idx = node.skip_idx;
            continue;
        }

        // Decode left_child from min[3]
        int left_child = __float_as_int(node.min[3]);

        if (left_child == -1) {
            // Leaf node — min[3] == -1, file_offset_id stored in max[3]
            uint32_t id_bits = __float_as_uint(node.max[3]);
            uint64_t file_offset = (uint64_t)id_bits;
            if (file_offset > 0) {
                hits[nhit++] = file_offset;
            }
            node_idx = node.skip_idx;
        } else {
            // Interior node — descend to left child
            node_idx = left_child;
        }
    }

    hit_counts[tid] = nhit;
}

// ── C launcher (called from Rust via FFI) ────────────────────────────────────
extern "C" void
engram_launch_bvh_traverse(const LBVHNode *d_nodes,
                           const Ray      *d_rays,
                           uint64_t       *d_hits,
                           int            *d_counts,
                           int             num_rays,
                           int             num_nodes,
                           int             max_hits) {
    int threads = 128;
    int blocks  = (num_rays + threads - 1) / threads;
    engram_bvh_traverse<<<blocks, threads>>>(
        d_nodes, d_rays, d_hits, d_counts, num_rays, num_nodes, max_hits);
    cudaDeviceSynchronize();
}
