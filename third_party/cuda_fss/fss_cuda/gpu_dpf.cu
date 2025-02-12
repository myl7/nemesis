#include "dpf.h"
#include "utils.h"

#include <stdio.h>
#include <stdlib.h>
#include <bit>
#include <vector>
#include <sys/random.h>

__device__
    RingElement
    re_get(RingElement x, u_int8_t pos)
{
    return (x >> pos) & 1;
}

__device__
    RingElement
    re_set(RingElement x, u_int8_t pos, u_int8_t value)
{
    return (x & (~((RingElement(1)) << pos))) ^ (RingElement(value) << pos);
}

__device__ void dpf_prf(const AES_KEY *k, const u_int8_t *seed, u_int8_t s[2][16], u_int8_t t[2])
{
    u_int8_t prf_out[DPF_PRF_LEN * 16];
    gpu_prf_gen<DPF_PRF_LEN>(seed, k, prf_out);

    memcpy(&s[0], prf_out, 16);
    memcpy(&s[1], &prf_out[16], 16);
    t[0] = s[0][0] & 1;
    s[0][0] ^= t[0];
    t[1] = s[1][0] & 1;
    s[1][0] ^= t[1];
}

__device__
    RingElement
    dpf_cvt(u_int8_t *seed)
{
    RingElement rst;
    memcpy(&rst, seed, sizeof(RingElement));
    return rst;
}

__device__ void gen_worker(RingElement alpha, RingElement beta, DpfKey *key_0, DpfKey *key_1, const u_int8_t *s)
{
    AES_KEY k[DPF_PRF_LEN];
    gpu_prf_init<DPF_PRF_LEN>(gpu_prfkey, k);

    u_int8_t t_0[2], t_1[2];

    u_int8_t cur_s0[2][16];
    u_int8_t cur_s1[2][16];
    u_int8_t *last_s[2];

    last_s[0] = cur_s0[0];
    last_s[1] = cur_s1[0];
    memcpy(last_s[0], s, 16);
    memcpy(last_s[1], &s[16], 16);
    u_int8_t last_t1 = last_s[1][0] & 1;
    last_s[0][0] ^= (last_s[0][0] & 1) ^ last_t1 ^ 1;

    memcpy(key_0->s, last_s[0], 16);
    memcpy(key_1->s, last_s[1], 16);

    last_s[0][0] ^= (last_s[0][0] & 1);
    last_s[1][0] ^= (last_s[1][0] & 1);

    RingElement rev_alpha = gpu_brev(alpha);
    for (int i = 0; i < RE_BIT; ++i)
    {
        u_int8_t alpha_bit = rev_alpha & 1;
        rev_alpha >>= 1;

        dpf_prf(k, last_s[0], cur_s0, t_0);
        dpf_prf(k, last_s[1], cur_s1, t_1);

        gpu_xor128(cur_s0[1 - alpha_bit], cur_s1[1 - alpha_bit], key_0->cw_s[i]);
        memcpy(key_1->cw_s[i], key_0->cw_s[i], 16);
        u_int8_t cw_t[2] = {(u_int8_t)(t_0[0] ^ t_1[0] ^ alpha_bit ^ 1), (u_int8_t)(t_0[1] ^ t_1[1] ^ alpha_bit)};

        key_0->cw_t0 = re_set(key_0->cw_t0, i, cw_t[0]);
        key_0->cw_t1 = re_set(key_0->cw_t1, i, cw_t[1]);
        key_1->cw_t0 = re_set(key_1->cw_t0, i, cw_t[0]);
        key_1->cw_t1 = re_set(key_1->cw_t1, i, cw_t[1]);

        last_s[0] = cur_s0[alpha_bit];
        last_s[1] = cur_s1[alpha_bit];
        gpu_xor128(last_s[last_t1], key_0->cw_s[i], last_s[last_t1]);
        last_t1 = t_1[alpha_bit] ^ (last_t1 * cw_t[alpha_bit]);
    }

    // printf("gen: %u %u\n",s[0],s[16]);
    key_0->last_cw = (1 - (last_t1 << 1)) * (beta + dpf_cvt(last_s[1]) - dpf_cvt(last_s[0]));
    key_1->last_cw = key_0->last_cw;
}

__device__ void eval_worker(RingElement x, const DpfKey *key, u_int8_t b, RingElement *rst)
{
    AES_KEY k[DPF_PRF_LEN];
    gpu_prf_init<DPF_PRF_LEN>(gpu_prfkey, k);

    u_int8_t cur_s[2][16];
    u_int8_t *last_s = cur_s[0];
    memcpy(last_s, key->s, 16);

    u_int8_t t[2];
    u_int8_t d = 0;
    u_int8_t last_t = last_s[0] & 1;
    // printf("eval: %u\n",last_s[0]);
    last_s[0] ^= last_t;

    RingElement rev_x = gpu_brev(x);
    for (int i = 0; i < RE_BIT; ++i)
    {
        // printf("%d %u %lu%lu\n",i,last_t,*(RingElement*)last_s,((RingElement*)(last_s))[1]);
        u_int8_t x_bit = rev_x & 1;
        rev_x >>= 1;
        dpf_prf(k, last_s, cur_s, t);
        if (last_t != 0)
        {
            gpu_xor128(cur_s[0], key->cw_s[i], cur_s[0]);
            t[0] = t[0] ^ re_get(key->cw_t0, i);
            gpu_xor128(cur_s[1], key->cw_s[i], cur_s[1]);
            t[1] = t[1] ^ re_get(key->cw_t1, i);
        }
        // if (d != x_bit) {
        //     d ^= last_t;
        //     if (last_t) {
        //         for (int j = 0; j < 16; ++j) {
        //             last_s[j] = ~last_s[j];
        //         }
        //     }
        // }
        last_s = cur_s[x_bit];
        last_t = t[x_bit];
    }
    // if (d) {
    //     if (last_t) {
    //         for (int j = 0; j < 16; ++j) {
    //             last_s[j] = ~last_s[j];
    //         }
    //     }
    // }
    *rst = (1 - (b << 1)) * (dpf_cvt(last_s) + last_t * key->last_cw);
}

__global__ void gen_assign(const RingElement *alpha, const RingElement *beta, DpfKey *key_0, DpfKey *key_1, u_int8_t *s, size_t num)
{
    size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < num)
    {
        gen_worker(alpha[idx], beta[idx], &key_0[idx], &key_1[idx], &s[2 * 16 * idx]);
    }
}

__global__ void eval_assign(const RingElement *x, const DpfKey *key, const u_int8_t b, RingElement *rst, size_t num)
{
    size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < num)
    {
        eval_worker(x[idx], &key[idx], b, &rst[idx]);
    }
}

__global__ void rep_eval_assign(const RingElement *x, const DpfKey *key, const u_int8_t b, RingElement *rst, size_t x_num)
{
    // size_t tb_for_x = (x_num + blockDim.x - 1) / blockDim.x;
    size_t x_idx = blockIdx.x * blockDim.x + threadIdx.x;
    size_t key_idx = blockIdx.y;
    size_t rst_idx = key_idx * x_num + x_idx;
    if (x_idx < x_num)
    {
        eval_worker(x[x_idx], &key[key_idx], b, &rst[rst_idx]);
    }
}

void batch_gen(const RingElement *alpha_arr, const RingElement *beta_arr, DpfKey *key0_arr, DpfKey *key1_arr, size_t num)
{
    RingElement *alpha_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&alpha_arr_gpu, num * sizeof(RingElement)));
    cudaMemcpy(alpha_arr_gpu, alpha_arr, num * sizeof(RingElement), cudaMemcpyHostToDevice);

    RingElement *beta_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&beta_arr_gpu, num * sizeof(RingElement)));
    cudaMemcpy(beta_arr_gpu, beta_arr, num * sizeof(RingElement), cudaMemcpyHostToDevice);
    DpfKey *key_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&key_arr_gpu, 2 * num * sizeof(DpfKey)));

    int rlen = 2 * 16 * num;
    u_int8_t *s_arr = (u_int8_t *)malloc(rlen);
    assert(getrandom(s_arr, rlen, GRND_RANDOM) == rlen);

    u_int8_t *s_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&s_arr_gpu, rlen));
    cudaMemcpy(s_arr_gpu, s_arr, rlen, cudaMemcpyHostToDevice);

    dim3 thread_dim(DPF_TB_N);
    dim3 block_dim((num + DPF_TB_N - 1) / DPF_TB_N);
    gen_assign<<<block_dim, thread_dim>>>(alpha_arr_gpu, beta_arr_gpu, key_arr_gpu, &key_arr_gpu[num], s_arr_gpu, num);

    cudaMemcpy(key0_arr, key_arr_gpu, num * sizeof(DpfKey), cudaMemcpyDeviceToHost);
    cudaMemcpy(key1_arr, &key_arr_gpu[num], num * sizeof(DpfKey), cudaMemcpyDeviceToHost);
    free(s_arr);
    cudaFree(alpha_arr_gpu);
    alpha_arr_gpu = NULL;
    cudaFree(beta_arr_gpu);
    beta_arr_gpu = NULL;
    cudaFree(key_arr_gpu);
    key_arr_gpu = NULL;
    cudaFree(s_arr_gpu);
    s_arr_gpu = NULL;
}

void batch_eval(const RingElement *x_arr, const DpfKey *key_arr, const u_int8_t b, RingElement *rst_arr, size_t num)
{
    RingElement *x_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&x_arr_gpu, num * sizeof(RingElement)));
    cudaMemcpy(x_arr_gpu, x_arr, num * sizeof(RingElement), cudaMemcpyHostToDevice);

    DpfKey *key_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&key_arr_gpu, num * sizeof(DpfKey)));
    cudaMemcpy(key_arr_gpu, key_arr, num * sizeof(DpfKey), cudaMemcpyHostToDevice);

    RingElement *rst_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&rst_arr_gpu, num * sizeof(RingElement)));

    dim3 thread_dim(DPF_TB_N);
    dim3 block_dim((num + DPF_TB_N - 1) / DPF_TB_N);
    eval_assign<<<block_dim, thread_dim>>>(x_arr_gpu, key_arr_gpu, b, rst_arr_gpu, num);

    cudaMemcpy(rst_arr, rst_arr_gpu, num * sizeof(RingElement), cudaMemcpyDeviceToHost);
    cudaFree(x_arr_gpu);
    x_arr_gpu = NULL;
    cudaFree(key_arr_gpu);
    key_arr_gpu = NULL;
    cudaFree(rst_arr_gpu);
    rst_arr_gpu = NULL;
}

void rep_eval(const RingElement *x_arr, const DpfKey *key_arr, u_int8_t b, RingElement *rst_arr, size_t key_num, size_t x_num)
{
    RingElement *x_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&x_arr_gpu, x_num * sizeof(RingElement)));
    cudaMemcpy(x_arr_gpu, x_arr, x_num * sizeof(RingElement), cudaMemcpyHostToDevice);

    DpfKey *key_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&key_arr_gpu, key_num * sizeof(DpfKey)));
    cudaMemcpy(key_arr_gpu, key_arr, key_num * sizeof(DpfKey), cudaMemcpyHostToDevice);

    RingElement *rst_arr_gpu;
    gpuErrchk(cudaMalloc((void **)&rst_arr_gpu, x_num * key_num * sizeof(RingElement)));

    size_t tb_for_x = (x_num + DPF_TB_N - 1) / DPF_TB_N;
    dim3 thread_dim(DPF_TB_N);
    dim3 block_dim(tb_for_x, key_num);
    rep_eval_assign<<<block_dim, thread_dim>>>(x_arr_gpu, key_arr_gpu, b, rst_arr_gpu, x_num);

    cudaMemcpy(rst_arr, rst_arr_gpu, x_num * key_num * sizeof(RingElement), cudaMemcpyDeviceToHost);
    cudaFree(x_arr_gpu);
    x_arr_gpu = NULL;
    cudaFree(key_arr_gpu);
    key_arr_gpu = NULL;
    cudaFree(rst_arr_gpu);
    rst_arr_gpu = NULL;
}

#ifndef HNVDEBUG
#include <pybind11/pybind11.h>
#include <pybind11/numpy.h>

namespace py = pybind11;

std::vector<py::array_t<u_int8_t>> gen_wrapper(
    py::array_t<RingElement, py::array::c_style | py::array::forcecast> alpha_np,
    py::array_t<RingElement, py::array::c_style | py::array::forcecast> beta_np,
    size_t num)
{
    auto key0_np = py::array_t<u_int8_t>(sizeof(DpfKey) * num);
    auto key1_np = py::array_t<u_int8_t>(sizeof(DpfKey) * num);

    RingElement *alpha_arr = static_cast<RingElement *>(alpha_np.request().ptr);
    RingElement *beta_arr = static_cast<RingElement *>(beta_np.request().ptr);
    DpfKey *key0_arr = static_cast<DpfKey *>(key0_np.request().ptr);
    DpfKey *key1_arr = static_cast<DpfKey *>(key1_np.request().ptr);

    batch_gen(alpha_arr, beta_arr, key0_arr, key1_arr, num);
    CHECK_LAST_CUDA_ERROR();
    return {key0_np, key1_np};
}

py::array_t<RingElement> rep_eval_wrapper(
    const py::array_t<u_int8_t, py::array::c_style | py::array::forcecast> key_np,
    const py::array_t<RingElement, py::array::c_style | py::array::forcecast> x_np,
    const size_t key_num,
    const size_t x_num,
    const u_int8_t b)
{
    auto rst_np = py::array_t<RingElement, py::array::c_style>(key_num * x_num);
    const DpfKey *key_arr = static_cast<DpfKey *>(key_np.request().ptr);
    const RingElement *x_arr = static_cast<RingElement *>(x_np.request().ptr);
    RingElement *rst_arr = static_cast<RingElement *>(rst_np.request().ptr);
    rep_eval(x_arr, key_arr, b, rst_arr, key_num, x_num);
    CHECK_LAST_CUDA_ERROR();
    return rst_np.reshape({key_num, x_num});
}

PYBIND11_MODULE(TORCH_EXTENSION_NAME, m)
{
    m.def("gen_wrapper", &gen_wrapper);
    m.def("rep_eval_wrapper", &rep_eval_wrapper);
}
#endif

#ifdef HNVDEBUG
void simple_dpf_test()
{
    int num = 1024 * 1024;
    RingElement *alpha_arr = (RingElement *)malloc(sizeof(RingElement) * num);
    RingElement *beta_arr = (RingElement *)malloc(sizeof(RingElement) * num);
    DpfKey *key0_arr = (DpfKey *)malloc(sizeof(DpfKey) * num);
    DpfKey *key1_arr = (DpfKey *)malloc(sizeof(DpfKey) * num);

    assert(getrandom(alpha_arr, sizeof(RingElement) * num, GRND_RANDOM) == sizeof(RingElement) * num);
    assert(getrandom(beta_arr, sizeof(RingElement) * num, GRND_RANDOM) == sizeof(RingElement) * num);

    batch_gen(alpha_arr, beta_arr, key0_arr, key1_arr, num);

    RingElement *x_arr = (RingElement *)malloc(sizeof(RingElement) * num);
    assert(getrandom(x_arr, sizeof(RingElement) * num, GRND_RANDOM) == sizeof(RingElement) * num);
    srand(42);
    for (int i = 0; i < num; ++i)
    {
        if (rand() % 2 == 1)
        {
            x_arr[i] = alpha_arr[i];
        }
    }

    RingElement *rst0_arr = (RingElement *)malloc(sizeof(RingElement) * num);
    RingElement *rst1_arr = (RingElement *)malloc(sizeof(RingElement) * num);
    struct timespec t_0, t_1;
    clock_gettime(CLOCK_REALTIME, &t_0);
    batch_eval(x_arr, key0_arr, (u_int8_t)0, rst0_arr, num);
    batch_eval(x_arr, key1_arr, (u_int8_t)1, rst1_arr, num);
    clock_gettime(CLOCK_REALTIME, &t_1);

    CHECK_LAST_CUDA_ERROR();

    for (int i = 0; i < num; ++i)
    {
        // printf("%lu %lu %lu %lu\n",x_arr[i],alpha_arr[i],beta_arr[i],rst_arr[2*i]+rst_arr[2*i+1]);
        assert(rst0_arr[i] + rst1_arr[i] == (x_arr[i] == alpha_arr[i] ? beta_arr[i] : RingElement(0)));
    }

    printf("keysize %lu\n", sizeof(DpfKey) * 2 * num);
    printf("eval speed %lf dpf/s\n", (2 * num) / ((t_1.tv_sec - t_0.tv_sec) + double(t_1.tv_nsec - t_0.tv_nsec) / 1000000000));
    printf("simple dpf test pass!\n");
}

void rep_dpf_test()
{
    size_t x_num = 7833;
    size_t key_num = 235;

    RingElement *alpha_arr = (RingElement *)malloc(sizeof(RingElement) * key_num);
    RingElement *beta_arr = (RingElement *)malloc(sizeof(RingElement) * key_num);
    DpfKey *key0_arr = (DpfKey *)malloc(sizeof(DpfKey) * key_num);
    DpfKey *key1_arr = (DpfKey *)malloc(sizeof(DpfKey) * key_num);

    assert(getrandom(alpha_arr, sizeof(RingElement) * key_num, GRND_RANDOM) == sizeof(RingElement) * key_num);
    assert(getrandom(beta_arr, sizeof(RingElement) * key_num, GRND_RANDOM) == sizeof(RingElement) * key_num);

    batch_gen(alpha_arr, beta_arr, key0_arr, key1_arr, key_num);

    RingElement *x_arr = (RingElement *)malloc(sizeof(RingElement) * x_num);
    assert(getrandom(x_arr, sizeof(RingElement) * x_num, GRND_RANDOM) == sizeof(RingElement) * x_num);
    srand(42);
    for (int i = 0; i < x_num; ++i)
    {
        if (rand() % 2 == 1)
        {
            x_arr[i] = alpha_arr[i % key_num];
        }
    }

    RingElement *rst0_arr = (RingElement *)malloc(sizeof(RingElement) * x_num * key_num);
    RingElement *rst1_arr = (RingElement *)malloc(sizeof(RingElement) * x_num * key_num);
    struct timespec t_0, t_1;
    clock_gettime(CLOCK_REALTIME, &t_0);
    rep_eval(x_arr, key0_arr, (u_int8_t)0, rst0_arr, key_num, x_num);
    rep_eval(x_arr, key1_arr, (u_int8_t)1, rst1_arr, key_num, x_num);
    clock_gettime(CLOCK_REALTIME, &t_1);

    CHECK_LAST_CUDA_ERROR();

    for (int i = 0; i < x_num * key_num; ++i)
    {
        // printf("%lu %lu %lu %lu\n",x_arr[i],alpha_arr[i],beta_arr[i],rst_arr[2*i]+rst_arr[2*i+1]);
        // printf("%lu,%lu,%u,%u,%u,%u\n",i/x_num,i%x_num,x_arr[i%x_num],alpha_arr[i/x_num],rst0_arr[i]+rst1_arr[i],beta_arr[i/x_num]);
        assert(rst0_arr[i] + rst1_arr[i] == (x_arr[i % x_num] == alpha_arr[i / x_num] ? beta_arr[i / x_num] : RingElement(0)));
    }

    printf("keysize %lu\n", sizeof(DpfKey));
    printf("eval speed %lf dpf/s\n", (2 * key_num * x_num) / ((t_1.tv_sec - t_0.tv_sec) + double(t_1.tv_nsec - t_0.tv_nsec) / 1000000000));
    printf("rep dpf test pass!\n");
}

int main()
{
    simple_dpf_test();
    rep_dpf_test();
    // printf("DpfKey size: %lu\n",sizeof(DpfKey));
    return 0;
}
#endif
