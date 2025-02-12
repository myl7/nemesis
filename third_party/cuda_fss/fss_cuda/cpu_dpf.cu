#include "dpf.h"
#include "utils.h"

#include <stdio.h>
#include <stdlib.h>
#include <bit>
#include <vector>
#include <sys/random.h>

RingElement re_get(RingElement x, u_int8_t pos)
{
    return (x >> pos) & 1;
}

RingElement re_set(RingElement x, u_int8_t pos, u_int8_t value)
{
    return (x & (~((RingElement(1)) << pos))) ^ (RingElement(value) << pos);
}

void dpf_prf(const AES_KEY *k, const u_int8_t *seed, u_int8_t s[2][16], u_int8_t t[2])
{
    u_int8_t prf_out[DPF_PRF_LEN * 16];
    cpu_prf_gen<DPF_PRF_LEN>(seed, k, prf_out);

    memcpy(&s[0], prf_out, 16);
    memcpy(&s[1], &prf_out[16], 16);
    t[0] = s[0][0] & 1;
    s[0][0] ^= t[0];
    t[1] = s[1][0] & 1;
    s[1][0] ^= t[1];
}

RingElement dpf_cvt(u_int8_t *seed)
{
    RingElement rst;
    memcpy(&rst, seed, sizeof(RingElement));
    return rst;
}

void gen_worker(RingElement alpha, RingElement beta, DpfKey *key_0, DpfKey *key_1, const u_int8_t *s)
{
    AES_KEY k[DPF_PRF_LEN];
    cpu_prf_init<DPF_PRF_LEN>(cpu_prfkey, k);

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

    RingElement rev_alpha = cpu_brev(alpha);
    for (int i = 0; i < RE_BIT; ++i)
    {
        u_int8_t alpha_bit = rev_alpha & 1;
        rev_alpha >>= 1;

        dpf_prf(k, last_s[0], cur_s0, t_0);
        dpf_prf(k, last_s[1], cur_s1, t_1);

        cpu_xor128(cur_s0[1 - alpha_bit], cur_s1[1 - alpha_bit], key_0->cw_s[i]);
        memcpy(key_1->cw_s[i], key_0->cw_s[i], 16);
        u_int8_t cw_t[2] = {(u_int8_t)(t_0[0] ^ t_1[0] ^ alpha_bit ^ 1), (u_int8_t)(t_0[1] ^ t_1[1] ^ alpha_bit)};

        key_0->cw_t0 = re_set(key_0->cw_t0, i, cw_t[0]);
        key_0->cw_t1 = re_set(key_0->cw_t1, i, cw_t[1]);
        key_1->cw_t0 = re_set(key_1->cw_t0, i, cw_t[0]);
        key_1->cw_t1 = re_set(key_1->cw_t1, i, cw_t[1]);

        last_s[0] = cur_s0[alpha_bit];
        last_s[1] = cur_s1[alpha_bit];
        cpu_xor128(last_s[last_t1], key_0->cw_s[i], last_s[last_t1]);
        last_t1 = t_1[alpha_bit] ^ (last_t1 * cw_t[alpha_bit]);
    }

    // printf("gen: %u %u\n",s[0],s[16]);
    key_0->last_cw = (1 - (last_t1 << 1)) * (beta + dpf_cvt(last_s[1]) - dpf_cvt(last_s[0]));
    key_1->last_cw = key_0->last_cw;
}

void eval_worker(RingElement x, const DpfKey *key, u_int8_t b, RingElement *rst)
{
    AES_KEY k[DPF_PRF_LEN];
    cpu_prf_init<DPF_PRF_LEN>(cpu_prfkey, k);

    u_int8_t cur_s[2][16];
    u_int8_t *last_s = cur_s[0];
    memcpy(last_s, key->s, 16);

    u_int8_t t[2];
    u_int8_t last_t = last_s[0] & 1;
    // printf("eval: %u\n",last_s[0]);
    last_s[0] ^= last_t;

    RingElement rev_x = cpu_brev(x);
    for (int i = 0; i < RE_BIT; ++i)
    {
        // printf("%d %u %lu%lu\n",i,last_t,*(RingElement*)last_s,((RingElement*)(last_s))[1]);
        u_int8_t x_bit = rev_x & 1;
        rev_x >>= 1;
        dpf_prf(k, last_s, cur_s, t);
        if (last_t != 0)
        {
            cpu_xor128(cur_s[0], key->cw_s[i], cur_s[0]);
            t[0] = t[0] ^ re_get(key->cw_t0, i);
            cpu_xor128(cur_s[1], key->cw_s[i], cur_s[1]);
            t[1] = t[1] ^ re_get(key->cw_t1, i);
        }
        last_s = cur_s[x_bit];
        last_t = t[x_bit];
    }
    *rst = (1 - (b << 1)) * (dpf_cvt(last_s) + last_t * key->last_cw);
}

void batch_gen(const RingElement *alpha_arr, const RingElement *beta_arr, DpfKey *key0_arr, DpfKey *key1_arr, size_t num)
{
    int rlen = 2 * 16 * num;
    u_int8_t *s_arr = (u_int8_t *)malloc(rlen);
    assert(getrandom(s_arr, rlen, GRND_RANDOM) == rlen);

    for (size_t i = 0; i < num; ++i)
    {
        gen_worker(alpha_arr[i], beta_arr[i], &key0_arr[i], &key1_arr[i], &s_arr[2 * 16 * i]);
    }

    free(s_arr);
}

void batch_eval(const RingElement *x_arr, const DpfKey *key_arr, const u_int8_t b, RingElement *rst_arr, size_t num)
{
    for (size_t i = 0; i < num; ++i)
    {
        eval_worker(x_arr[i], &key_arr[i], b, &rst_arr[i]);
    }
}

void rep_eval(const RingElement *x_arr, const DpfKey *key_arr, u_int8_t b, RingElement *rst_arr, size_t key_num, size_t x_num)
{
    for (size_t i = 0; i < key_num; ++i)
    {
        for (size_t j = 0; j < x_num; ++j)
        {
            eval_worker(x_arr[j], &key_arr[i], b, &rst_arr[i * x_num + j]);
        }
    }
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

int main()
{
    simple_dpf_test();
    return 0;
}

#endif
