from setuptools import setup, Extension
from torch.utils.cpp_extension import BuildExtension, CUDAExtension
from torch.utils import cpp_extension

setup(
    name="fss",
    ext_modules=[
        CUDAExtension(
            name="fss_cpu.cpu_dpf",
            sources=["fss_cuda/cpu_dpf.cu"],
            extra_compile_args=["-std=c++17"],
        ),
        # CUDAExtension(
        #     name="fss_cpu.cpu_cpdpf",
        #     sources=["fss_cuda/cpu_cpdpf.cu"],
        #     extra_compile_args=["-std=c++17"],
        # ),
        # CUDAExtension(
        #     name="fss_cpu.cpu_cpdcf",
        #     sources=["fss_cuda/cpu_cpdcf.cu"],
        #     extra_compile_args=["-std=c++17"],
        # ),
        CUDAExtension(
            name="fss_gpu.gpu_dpf",
            sources=["fss_cuda/gpu_dpf.cu"],
            extra_compile_args=["-std=c++17"],
        ),
        # CUDAExtension(
        #     name="fss_gpu.gpu_cpdpf",
        #     sources=["fss_cuda/gpu_cpdpf.cu"],
        #     extra_compile_args=["-std=c++17"],
        # ),
        # CUDAExtension(
        #     name="fss_gpu.gpu_cpdcf",
        #     sources=["fss_cuda/gpu_cpdcf.cu"],
        #     extra_compile_args=["-std=c++17"],
        # ),
    ],
    cmdclass={'build_ext':cpp_extension.BuildExtension}
)
