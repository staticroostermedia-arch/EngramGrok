// Phase 8 — OptiX Any-Hit Program (Engram)
// Collects ALL AABB hits for KNN. optixIgnoreIntersection() keeps traversal going.

#include <optix.h>

struct EngramLaunchParams {
    float3*                  d_query_positions;
    OptixTraversableHandle   traversable;
    unsigned long long*      d_hit_list;
    unsigned int*            d_hit_counts;
    unsigned int             max_hits_per_query;
    const float*             d_aabb_data;   // packed [minX,minY,minZ,maxX,maxY,maxZ] × n
};

extern "C" __constant__ EngramLaunchParams params;

extern "C" __global__ void __anyhit__engram_collect() {
    unsigned int hit_count  = optixGetPayload_0();
    unsigned int write_base = optixGetPayload_1();

    if (hit_count < params.max_hits_per_query) {
        const unsigned int prim_idx = optixGetPrimitiveIndex();
        params.d_hit_list[(unsigned long long)write_base + hit_count] =
            (unsigned long long)(prim_idx + 1u);   // 1-based to match CPU BVH
        optixSetPayload_0(hit_count + 1u);
    }

    optixIgnoreIntersection();  // CRITICAL: continue traversal for KNN
}
