// Phase 8 — OptiX Ray Generation Program (Engram)
// Identical logic to CodeLand's optix_rg.cu; struct name uses "Engram" prefix.

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

extern "C" __global__ void __raygen__engram_knn() {
    const unsigned int qidx = optixGetLaunchIndex().x;

    const float3 origin = params.d_query_positions[qidx];
    const float3 dir    = make_float3(0.5773502692f, 0.5773502692f, 0.5773502692f);

    unsigned int p0 = 0u;
    unsigned int p1 = qidx * params.max_hits_per_query;

    optixTrace(
        params.traversable,
        origin, dir,
        0.0f, 1e16f, 0.0f,
        OptixVisibilityMask(255),
        OPTIX_RAY_FLAG_NONE,
        0, 1, 0,
        p0, p1
    );

    params.d_hit_counts[qidx] = p0;
}
