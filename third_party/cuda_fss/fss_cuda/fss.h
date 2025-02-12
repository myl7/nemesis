#pragma once

// #define RE_BIT (64)
#define RE_BIT (32)

#include "prf/prf.h"
// #include "utils.h"

#if RE_BIT == 64
typedef u_int64_t RingElement;
#elif RE_BIT == 32
typedef u_int32_t RingElement;
#endif

#ifndef HNVDEBUG
torch::TensorOptions conf_ten = torch::TensorOptions().dtype(torch::kUInt8).layout(torch::kStrided).device(torch::kCPU).requires_grad(false);
#endif
