#pragma once

#ifndef HNVDEBUG
#include <torch/extension.h>
#endif

#include <cuda.h>
#include <cuda_runtime.h>

#include "fss.h"

#define DPF_TB_N (512)
#define DPF_PRF_LEN (2)

typedef struct
{
    u_int8_t s[16];
    u_int8_t cw_s[RE_BIT][16];
    RingElement cw_t0;
    RingElement cw_t1;
    RingElement last_cw;
} DpfKey;

typedef struct
{
    u_int8_t cw_s[RE_BIT][16];
    RingElement cw_t0;
    RingElement cw_t1;
    RingElement last_cw;
} CpDpfKey;

#if (DPF_PRF_LEN == 2)
#define PRF_KEY {95, 76, 249, 68, 245, 241, 202, 251, 122, 192, 21, 145, 109, 49, 201, 39, 99, 130, 245, 7, 225, 187, 162, 116, 13, 225, 140, 145, 5, 178, 125, 80}
__device__ const u_int8_t gpu_prfkey[32] = PRF_KEY;
const u_int8_t cpu_prfkey[32] = PRF_KEY;
__device__ const u_int8_t cpprf_key[32] = PRF_KEY;
#endif
