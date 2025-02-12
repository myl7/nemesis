import time
import random
import hashlib

# version: 1.0
# date: 20231007

def example():
    import torch
    from fss_cpu import cpu_dpf
    from fss_gpu import gpu_dpf
    import numpy as np
    np.seterr(over='ignore')
    from random import randint
    RingElement = np.uint32
    RE_BYTE = RingElement(0).itemsize

    # some random parameter
    key_num = 77 * 10 ** 5
    x_num = 10 ** 8

    # generate dpf key with cpu
    t1 = time.time()
    for _ in range(13):
        alpha = np.frombuffer(np.random.bytes(RE_BYTE*key_num),dtype=RingElement)
        beta = np.frombuffer(np.random.bytes(RE_BYTE*key_num),dtype=RingElement)
        key_0, key_1 = gpu_dpf.gen_wrapper(alpha,beta,key_num)
    print("time for key generation:",time.time()-t1)
    exit(0)

    # evaluating dpf keys on gpu
    x = np.frombuffer(np.random.bytes(RE_BYTE*x_num),dtype=RingElement).copy()
    assert(x_num>=key_num)
    for i in range(x_num):
        if randint(0,1) == 1:
            x[i] = alpha[i%key_num]

    # note that the keys of two servers are evaluated separately
    # t1 = time.time()
    rst_0 = gpu_dpf.rep_eval_wrapper(key_0,x,key_num,x_num,0)
    rst_1 = gpu_dpf.rep_eval_wrapper(key_1,x,key_num,x_num,1)

    # verify the result
    for i in range(key_num*x_num):
        assert(rst_0.reshape(-1)[i]+rst_1.reshape(-1)[i] == (beta[i//x_num] if x[i%x_num]==alpha[i//x_num] else RingElement(0)))
    # print("time for server 0:",time.time()-t1)

    # mk = int.from_bytes(random.randbytes(264),byteorder='little')
    # mk = int.from_bytes(random.randbytes(32),byteorder='little')

    # t1 = time.time()
    # sum_val = 0
    # for i in range(key_num):
    #     for j in range(x_num):
    #         sum_val = (sum_val + (rst_0[i][j] * mk) & ('\xff' * 32)) & ('\xff' * 32)
    # print("time for sum:",time.time()-t1)

    print("example test pass")

if __name__ == "__main__":
    example()
