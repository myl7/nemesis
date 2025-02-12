#pragma once

#include "../fss.h"
#include "aes_core.h"

template <int len>
__device__
void gpu_prf_init(const u_int8_t* key, AES_KEY* k){
    for (int i = 0; i < len; ++i){
        GPU_AES_set_encrypt_key(&key[16*i],128,&k[i]);
    }
    return;
}

template <int len>
__device__
void gpu_prf_gen(const u_int8_t* seed, const AES_KEY* k, u_int8_t* r) {
    for (int i = 0; i < len; ++i){
        GPU_AES_encrypt(seed,&r[i*16],&k[i]);
    }
    return;
}

template <int len>
void cpu_prf_init(const u_int8_t* key, AES_KEY* k){
    for (int i = 0; i < len; ++i){
        CPU_AES_set_encrypt_key(&key[16*i],128,&k[i]);
    }
    return;
}

template <int len>
void cpu_prf_gen(const u_int8_t* seed, const AES_KEY* k, u_int8_t* r) {
    for (int i = 0; i < len; ++i){
        CPU_AES_encrypt(seed,&r[i*16],&k[i]);
    }
    return;
}
