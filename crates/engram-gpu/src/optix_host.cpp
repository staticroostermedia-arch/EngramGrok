// Phase 8 — OptiX BVH Pipeline Host Code (Engram)
//
// Mirrors CodeLand's optix_host.cpp with Engram-prefixed symbols.
// Exposes a clean C ABI for Rust FFI from optix_pipeline.rs.
//
// Build gate: compiled only when OPTIX_SDK_AVAILABLE is defined (i.e. OPTIX_SDK_PATH set).
// Without the SDK this file compiles as pure stubs that return nullptr / -1.

#ifdef OPTIX_SDK_AVAILABLE

#include <optix.h>
#include <optix_stubs.h>
// REQUIRED: defines g_optixFunctionTable (the version-stamped function pointer table).
// optix_stubs.h only declares it — this header provides the one definition per link unit.
#include <optix_function_table_definition.h>
#include <cuda_runtime.h>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <vector>

// ── Helpers ──────────────────────────────────────────────────────────────────

#define OPTIX_CHECK_NULL(call)                                              \
    do {                                                                    \
        OptixResult res = (call);                                           \
        if (res != OPTIX_SUCCESS) {                                         \
            fprintf(stderr, "[Engram-OptiX] %s:%d — %s (%d)\n",            \
                    __FILE__, __LINE__, optixGetErrorString(res), (int)res);\
            return nullptr;                                                 \
        }                                                                   \
    } while(0)

// ── Launch parameters (must match kernels/optix_rg.cu + optix_ah.cu) ─────────

struct EngramLaunchParams {
    float3*                 d_query_positions;
    OptixTraversableHandle  traversable;
    unsigned long long*     d_hit_list;
    unsigned int*           d_hit_counts;
    unsigned int            max_hits_per_query;
    const float*            d_aabb_data;   // packed [minX,minY,minZ,maxX,maxY,maxZ] × n
};

// ── Pipeline state ────────────────────────────────────────────────────────────

struct EngramOptiXPipeline {
    OptixDeviceContext      context     = nullptr;
    OptixPipeline           pipeline    = nullptr;
    OptixShaderBindingTable sbt         = {};
    OptixTraversableHandle  traversable = 0;

    CUdeviceptr d_sbt_hitgroup  = 0;
    CUdeviceptr d_sbt_raygen    = 0;
    CUdeviceptr d_sbt_miss      = 0;
    CUdeviceptr d_gas_output    = 0;
    CUdeviceptr d_aabb_input    = 0;

    CUdeviceptr d_query_pos     = 0;
    CUdeviceptr d_hit_list      = 0;
    CUdeviceptr d_hit_counts    = 0;
    uint32_t    scratch_queries = 0;
    uint32_t    scratch_hits    = 0;
};

static void engram_optix_log(unsigned int level, const char* tag,
                              const char* msg, void*) {
    if (level <= 3) fprintf(stderr, "[Engram-OptiX:%s] %s\n", tag, msg);
}

// ── Public C API ─────────────────────────────────────────────────────────────

extern "C" {

EngramOptiXPipeline* engram_optix_init(
    const float* aabb_data,
    uint32_t     n_primitives,
    const char*  ptx_is,
    const char*  ptx_rg,
    const char*  ptx_ah,
    const char*  ptx_ms)
{
    auto* state = new EngramOptiXPipeline();

    OPTIX_CHECK_NULL(optixInit());

    cudaFree(nullptr);
    CUcontext cu_ctx = nullptr;
    cuCtxGetCurrent(&cu_ctx);

    OptixDeviceContextOptions ctx_opts = {};
    ctx_opts.logCallbackFunction = &engram_optix_log;
    ctx_opts.logCallbackLevel    = 3;
    OPTIX_CHECK_NULL(optixDeviceContextCreate(cu_ctx, &ctx_opts, &state->context));

    OptixModuleCompileOptions mod_opts = {};
    mod_opts.optLevel   = OPTIX_COMPILE_OPTIMIZATION_DEFAULT;
    mod_opts.debugLevel = OPTIX_COMPILE_DEBUG_LEVEL_NONE;

    OptixPipelineCompileOptions pipe_opts = {};
    pipe_opts.usesMotionBlur                   = 0;
    pipe_opts.traversableGraphFlags            = OPTIX_TRAVERSABLE_GRAPH_FLAG_ALLOW_SINGLE_GAS;
    pipe_opts.numPayloadValues                 = 2;
    pipe_opts.numAttributeValues               = 0;
    pipe_opts.usesPrimitiveTypeFlags           = OPTIX_PRIMITIVE_TYPE_FLAGS_CUSTOM;  // renamed in OptiX 9
    pipe_opts.pipelineLaunchParamsVariableName = "params";

    char log[4096]; size_t log_size = sizeof(log);

    // PTX text is valid SM-native virtual ISA. _optix_* intrinsics are resolved
    // by the OptiX 9.1 JIT at optixModuleCreate time — not by ptxas.
    // We compile with nvcc --ptx --gpu-architecture=compute_120 (native SM 12.0)
    // and ignore ptxas exit codes (CUDA 13 ptxas rejects _optix_* as unknown;
    // the PTX file is written before ptxas validation runs).
    auto compile_mod = [&](const char* ptx, OptixModule* mod) -> bool {
        log_size = sizeof(log);
        OptixResult r = optixModuleCreate(
            state->context, &mod_opts, &pipe_opts,
            ptx, strlen(ptx), log, &log_size, mod);
        if (r != OPTIX_SUCCESS) {
            fprintf(stderr, "[Engram-OptiX] Module compile failed: %s\n", log);
            return false;
        }
        return true;
    };

    OptixModule mod_is, mod_rg, mod_ah, mod_ms;
    if (!compile_mod(ptx_is, &mod_is) ||
        !compile_mod(ptx_rg, &mod_rg) ||
        !compile_mod(ptx_ah, &mod_ah) ||
        !compile_mod(ptx_ms, &mod_ms)) {
        delete state;
        return nullptr;
    }

    OptixProgramGroup pg_rg, pg_hg, pg_ms;

    {
        OptixProgramGroupDesc d = {};
        d.kind = OPTIX_PROGRAM_GROUP_KIND_RAYGEN;
        d.raygen.module = mod_rg;
        d.raygen.entryFunctionName = "__raygen__engram_knn";
        log_size = sizeof(log);
        OPTIX_CHECK_NULL(optixProgramGroupCreate(state->context, &d, 1, nullptr, log, &log_size, &pg_rg));
    }
    {
        OptixProgramGroupDesc d = {};
        d.kind = OPTIX_PROGRAM_GROUP_KIND_HITGROUP;
        d.hitgroup.moduleAH             = mod_ah;
        d.hitgroup.entryFunctionNameAH  = "__anyhit__engram_collect";
        d.hitgroup.moduleIS             = mod_is;
        d.hitgroup.entryFunctionNameIS  = "__intersection__engram_aabb";
        log_size = sizeof(log);
        OPTIX_CHECK_NULL(optixProgramGroupCreate(state->context, &d, 1, nullptr, log, &log_size, &pg_hg));
    }
    {
        OptixProgramGroupDesc d = {};
        d.kind = OPTIX_PROGRAM_GROUP_KIND_MISS;
        d.miss.module = mod_ms;
        d.miss.entryFunctionName = "__miss__engram_knn";
        log_size = sizeof(log);
        OPTIX_CHECK_NULL(optixProgramGroupCreate(state->context, &d, 1, nullptr, log, &log_size, &pg_ms));
    }

    OptixProgramGroup all_pgs[] = { pg_rg, pg_hg, pg_ms };
    OptixPipelineLinkOptions link_opts = {};
    link_opts.maxTraceDepth = 1;
    log_size = sizeof(log);
    OPTIX_CHECK_NULL(optixPipelineCreate(state->context, &pipe_opts, &link_opts,
                                         all_pgs, 3, log, &log_size, &state->pipeline));
    optixPipelineSetStackSize(state->pipeline, 2048, 2048, 2048, 1);

    // Build GAS
    const size_t aabb_bytes = (size_t)n_primitives * 6 * sizeof(float);
    cudaMalloc(reinterpret_cast<void**>(&state->d_aabb_input), aabb_bytes);
    cudaMemcpy(reinterpret_cast<void*>(state->d_aabb_input),
               aabb_data, aabb_bytes, cudaMemcpyHostToDevice);

    OptixBuildInput bi = {};
    bi.type = OPTIX_BUILD_INPUT_TYPE_CUSTOM_PRIMITIVES;
    bi.customPrimitiveArray.aabbBuffers   = &state->d_aabb_input;
    bi.customPrimitiveArray.numPrimitives = n_primitives;
    uint32_t flags[1] = { OPTIX_GEOMETRY_FLAG_NONE };
    bi.customPrimitiveArray.flags         = flags;
    bi.customPrimitiveArray.numSbtRecords = 1;

    OptixAccelBuildOptions accel_opts = {};
    accel_opts.buildFlags = OPTIX_BUILD_FLAG_ALLOW_COMPACTION | OPTIX_BUILD_FLAG_PREFER_FAST_TRACE;
    accel_opts.operation  = OPTIX_BUILD_OPERATION_BUILD;

    OptixAccelBufferSizes sizes = {};
    OPTIX_CHECK_NULL(optixAccelComputeMemoryUsage(state->context, &accel_opts, &bi, 1, &sizes));

    CUdeviceptr d_temp, d_out_full;
    cudaMalloc(reinterpret_cast<void**>(&d_temp),     sizes.tempSizeInBytes);
    cudaMalloc(reinterpret_cast<void**>(&d_out_full), sizes.outputSizeInBytes);

    OptixTraversableHandle h_full;
    OPTIX_CHECK_NULL(optixAccelBuild(state->context, nullptr, &accel_opts,
                                     &bi, 1,
                                     d_temp, sizes.tempSizeInBytes,
                                     d_out_full, sizes.outputSizeInBytes,
                                     &h_full, nullptr, 0));
    cudaFree(reinterpret_cast<void*>(d_temp));

    // Compact
    CUdeviceptr d_compact_size;
    cudaMalloc(reinterpret_cast<void**>(&d_compact_size), sizeof(size_t));
    OptixAccelEmitDesc emit = { d_compact_size, OPTIX_PROPERTY_TYPE_COMPACTED_SIZE };
    optixAccelBuild(state->context, nullptr, &accel_opts, &bi, 1,
                    d_temp, sizes.tempSizeInBytes,
                    d_out_full, sizes.outputSizeInBytes,
                    &h_full, &emit, 1);
    size_t compact_size = 0;
    cudaMemcpy(&compact_size, reinterpret_cast<void*>(d_compact_size),
               sizeof(size_t), cudaMemcpyDeviceToHost);
    cudaFree(reinterpret_cast<void*>(d_compact_size));

    cudaMalloc(reinterpret_cast<void**>(&state->d_gas_output), compact_size);
    OPTIX_CHECK_NULL(optixAccelCompact(state->context, nullptr,
                                       h_full, state->d_gas_output, compact_size,
                                       &state->traversable));
    cudaFree(reinterpret_cast<void*>(d_out_full));

    // SBT
    struct RaygenRecord  { char h[OPTIX_SBT_RECORD_HEADER_SIZE]; };
    struct MissRecord    { char h[OPTIX_SBT_RECORD_HEADER_SIZE]; };
    struct HgRecord      { char h[OPTIX_SBT_RECORD_HEADER_SIZE]; };

    RaygenRecord rg = {}; optixSbtRecordPackHeader(pg_rg, &rg);
    MissRecord   ms = {}; optixSbtRecordPackHeader(pg_ms, &ms);
    HgRecord     hg = {}; optixSbtRecordPackHeader(pg_hg, &hg);

    cudaMalloc(reinterpret_cast<void**>(&state->d_sbt_raygen),    sizeof(rg));
    cudaMalloc(reinterpret_cast<void**>(&state->d_sbt_miss),      sizeof(ms));
    cudaMalloc(reinterpret_cast<void**>(&state->d_sbt_hitgroup),  sizeof(hg));

    cudaMemcpy(reinterpret_cast<void*>(state->d_sbt_raygen),   &rg, sizeof(rg), cudaMemcpyHostToDevice);
    cudaMemcpy(reinterpret_cast<void*>(state->d_sbt_miss),     &ms, sizeof(ms), cudaMemcpyHostToDevice);
    cudaMemcpy(reinterpret_cast<void*>(state->d_sbt_hitgroup), &hg, sizeof(hg), cudaMemcpyHostToDevice);

    state->sbt.raygenRecord                = state->d_sbt_raygen;
    state->sbt.missRecordBase              = state->d_sbt_miss;
    state->sbt.missRecordStrideInBytes     = sizeof(MissRecord);
    state->sbt.missRecordCount             = 1;
    state->sbt.hitgroupRecordBase          = state->d_sbt_hitgroup;
    state->sbt.hitgroupRecordStrideInBytes = sizeof(HgRecord);
    state->sbt.hitgroupRecordCount         = 1;

    fprintf(stderr, "[Engram-OptiX] Pipeline ready: %u primitives, GAS=%.1fKB\n",
            n_primitives, compact_size / 1024.0f);
    return state;
}

int engram_optix_query(
    EngramOptiXPipeline* state,
    const float* positions,
    uint32_t     n_queries,
    uint32_t     max_hits,
    uint64_t*    hit_list_out,
    uint32_t*    hit_counts_out)
{
    if (!state || !state->pipeline) return -1;

    const size_t pos_bytes   = (size_t)n_queries * 3 * sizeof(float);
    const size_t hits_bytes  = (size_t)n_queries * max_hits * sizeof(uint64_t);
    const size_t count_bytes = (size_t)n_queries * sizeof(uint32_t);

    if (n_queries > state->scratch_queries || max_hits > state->scratch_hits) {
        if (state->d_query_pos)  cudaFree(reinterpret_cast<void*>(state->d_query_pos));
        if (state->d_hit_list)   cudaFree(reinterpret_cast<void*>(state->d_hit_list));
        if (state->d_hit_counts) cudaFree(reinterpret_cast<void*>(state->d_hit_counts));
        cudaMalloc(reinterpret_cast<void**>(&state->d_query_pos),   pos_bytes);
        cudaMalloc(reinterpret_cast<void**>(&state->d_hit_list),    hits_bytes);
        cudaMalloc(reinterpret_cast<void**>(&state->d_hit_counts),  count_bytes);
        state->scratch_queries = n_queries;
        state->scratch_hits    = max_hits;
    }

    cudaMemcpy(reinterpret_cast<void*>(state->d_query_pos),
               positions, pos_bytes, cudaMemcpyHostToDevice);
    cudaMemset(reinterpret_cast<void*>(state->d_hit_counts), 0, count_bytes);

    EngramLaunchParams params;
    params.d_query_positions  = reinterpret_cast<float3*>(state->d_query_pos);
    params.traversable        = state->traversable;
    params.d_hit_list         = reinterpret_cast<unsigned long long*>(state->d_hit_list);
    params.d_hit_counts       = reinterpret_cast<unsigned int*>(state->d_hit_counts);
    params.max_hits_per_query = max_hits;
    params.d_aabb_data        = reinterpret_cast<const float*>(state->d_aabb_input);

    CUdeviceptr d_params;
    cudaMalloc(reinterpret_cast<void**>(&d_params), sizeof(params));
    cudaMemcpy(reinterpret_cast<void*>(d_params), &params, sizeof(params), cudaMemcpyHostToDevice);

    OptixResult r = optixLaunch(state->pipeline, nullptr,
                                d_params, sizeof(params),
                                &state->sbt, n_queries, 1, 1);
    cudaFree(reinterpret_cast<void*>(d_params));
    if (r != OPTIX_SUCCESS) return -2;

    cudaDeviceSynchronize();

    cudaMemcpy(hit_list_out,   reinterpret_cast<void*>(state->d_hit_list),
               hits_bytes,  cudaMemcpyDeviceToHost);
    cudaMemcpy(hit_counts_out, reinterpret_cast<void*>(state->d_hit_counts),
               count_bytes, cudaMemcpyDeviceToHost);
    return 0;
}

void engram_optix_free(EngramOptiXPipeline* state) {
    if (!state) return;
    if (state->pipeline)        optixPipelineDestroy(state->pipeline);
    if (state->context)         optixDeviceContextDestroy(state->context);
    if (state->d_gas_output)    cudaFree(reinterpret_cast<void*>(state->d_gas_output));
    if (state->d_aabb_input)    cudaFree(reinterpret_cast<void*>(state->d_aabb_input));
    if (state->d_sbt_raygen)    cudaFree(reinterpret_cast<void*>(state->d_sbt_raygen));
    if (state->d_sbt_miss)      cudaFree(reinterpret_cast<void*>(state->d_sbt_miss));
    if (state->d_sbt_hitgroup)  cudaFree(reinterpret_cast<void*>(state->d_sbt_hitgroup));
    if (state->d_query_pos)     cudaFree(reinterpret_cast<void*>(state->d_query_pos));
    if (state->d_hit_list)      cudaFree(reinterpret_cast<void*>(state->d_hit_list));
    if (state->d_hit_counts)    cudaFree(reinterpret_cast<void*>(state->d_hit_counts));
    delete state;
}

} // extern "C"

#else // stubs

#include <cstdint>
#include <cstdio>
extern "C" {
void* engram_optix_init(const float*, uint32_t, const char*, const char*, const char*, const char*) {
    fprintf(stderr, "[Engram-OptiX] SDK not compiled in — using CPU BVH.\n");
    return nullptr;
}
int  engram_optix_query(void*, const float*, uint32_t, uint32_t, uint64_t*, uint32_t*) { return -1; }
void engram_optix_free(void*) {}
}

#endif
