// Phase 8 — OptiX Intersection Program (Engram)
//
// Custom IS program for AABB primitives in the Engram semantic manifold.
// Implements Condition 2: interior rays (query origin inside AABB) report t=0.0
// so that RT-Core traversal does not discard them, enabling KNN recall for
// queries whose 3D projection lands inside the concept's bounding sphere.
//
// GENESIS_SEED = 0x454E4752 ("ENGR") — matches engram-gpu/src/bvh.rs.
// AABB_RADIUS  = 200.0               — matches engram-core/src/index.rs.
//
// OptiX 9 note: optixGetPrimitiveAabb() was removed. AABB bounds are now
// fetched from the launch params d_aabb_data buffer via optixGetPrimitiveIndex().
//
// Compiled to PTX by build.rs:
//   nvcc --ptx --gpu-architecture=compute_86 -I${OPTIX_SDK_PATH}/include optix_intersect.cu

#include <optix.h>
#include <stdint.h>

// Must match the layout in optix_rg.cu, optix_ah.cu, and optix_host.cpp.
struct EngramLaunchParams {
    float3*                  d_query_positions;
    OptixTraversableHandle   traversable;
    unsigned long long*      d_hit_list;
    unsigned int*            d_hit_counts;
    unsigned int             max_hits_per_query;
    const float*             d_aabb_data;   // packed [minX,minY,minZ,maxX,maxY,maxZ] × n
};

extern "C" __constant__ EngramLaunchParams params;

extern "C" __global__ void __intersection__engram_aabb() {
    // Query ray in object space (same as world space — no instance transforms)
    const float3 ro = optixGetObjectRayOrigin();
    const float3 rd = optixGetObjectRayDirection();

    // Retrieve per-primitive AABB bounds from launch params (OptiX 9 compatible).
    // optixGetPrimitiveAabb() was removed after OptiX 7.3; use the primitive
    // index to look up bounds in the device-side aabb_data array instead.
    const uint32_t pi = optixGetPrimitiveIndex();
    const float* ab   = params.d_aabb_data + (size_t)pi * 6;
    // ab[0..2] = min XYZ,  ab[3..5] = max XYZ

    // Slab test (matches bvh_traverse.cu intersect_aabb logic)
    const float eps = 1e-8f;
    const float3 inv_rd = make_float3(
        fabsf(rd.x) > eps ? 1.0f / rd.x : (rd.x >= 0.0f ?  1e8f : -1e8f),
        fabsf(rd.y) > eps ? 1.0f / rd.y : (rd.y >= 0.0f ?  1e8f : -1e8f),
        fabsf(rd.z) > eps ? 1.0f / rd.z : (rd.z >= 0.0f ?  1e8f : -1e8f)
    );

    const float3 t0 = make_float3(
        (ab[0] - ro.x) * inv_rd.x,
        (ab[1] - ro.y) * inv_rd.y,
        (ab[2] - ro.z) * inv_rd.z
    );
    const float3 t1 = make_float3(
        (ab[3] - ro.x) * inv_rd.x,
        (ab[4] - ro.y) * inv_rd.y,
        (ab[5] - ro.z) * inv_rd.z
    );

    const float tmin = fmaxf(fmaxf(fminf(t0.x, t1.x), fminf(t0.y, t1.y)), fminf(t0.z, t1.z));
    const float tmax = fminf(fminf(fmaxf(t0.x, t1.x), fmaxf(t0.y, t1.y)), fmaxf(t0.z, t1.z));

    // Standard test: tmax >= 0 and tmax >= tmin
    if (tmax >= 0.0f && tmax >= tmin) {
        // Condition 2: interior ray (query origin inside AABB) → tmin < 0, tmax > 0
        // Override: report t=0 so RT Core registers the hit rather than discarding it.
        const float t_hit = fmaxf(0.0f, tmin);
        optixReportIntersection(t_hit, 0u);
    }
}
