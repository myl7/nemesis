# FSS Quick Start

This code is an adaption of [GPU-DPF](https://github.com/facebookresearch/GPU-DPF) from facebook research team.

20241021 update:

The following instruction is somewhat out-dated. I am now considering manage the nvidia cuda environment throught container, rather than conda that is used in this instruction. Conda still have some problem installing different cuda toolkits in different environments. The container has been used in some recent researches, for example, [Orca and SIGMA](https://github.com/mpc-msri/EzPC/tree/master/GPU-MPC).

However, given the very frustrating experience I had setting up the CUDA environment, I decide not to try the container approach. I am satisified with the conda solution used before, since I am the only one writing this CUDA code on the server, and I don't really need to work with different CUDA versions at the same time. I also use only one process to simulate all servers in MPC protocol in this experiment, therefore cannot benefit much from the easier deployment provided by the container. With that said, if you want to prevent conflict of projects requiring different CUDA versions, or easily deploy the same environment on different servers, then the container might be the right choice.

## Architecture

The FSS code is responsible for generating and evaluating FSS keys using either CPU or GPU. This module is intended to be called by python to accelerate the FSS-related computation. We use the extension functionality provided by pytorch to exchange data between Python and CUDA part.

## Environment

The GPU version of this code relies on Nvidia GPU (In our case, Nvidia RTX A6000 GPU) and Ubuntu 20.04, the dependencies can be installed by:

1. install the official version of nvidia driver through apt utility, and

2. manage the CUDA and pythroch utilities throught conda.

There are the detailed instructions:

### Install the official Nvidia driver:

Dell servers and ices servers have had these drivers. If you are on this two servers, you may safely skip this step.

[Click here for detailed instruction](https://ubuntu.com/server/docs/nvidia-drivers-installation). Just install the normal version, e.g. nvidia-driver-535 without any -server or -open suffix.

Also, we only need the GPU driver, we will install the cuda toolkit with conda. So, make sure you do not install the toolkit.

### Install Conda

Personally, I prefer anaconda, so [this is the detailed instruction for anaconda installation](https://docs.anaconda.com/free/anaconda/install/linux/). Miniconda works as well. so if you already have anaconda or miniconda installed, you can skip this step.

For some well known reason, you might want the use some mirror for downloading the packages in China. [Please check here](https://help.mirrors.cernet.edu.cn/anaconda/).

### Install Cuda and PyTorch

Make sure you have conda installed, and activate the conda environment.

Note that you might be in the default environment. If you don't want to install cuda environment (you probably don't), ```conda create --name <env_name> python=3.11``` (The FSS code is only tested under python3.11, there might be compatibility issue with other python versions) and activate the corresponding environment with ```conda activate <env_name>```.

[Check here for doemstic CUDA mirror](https://help.mirrors.cernet.edu.cn/anaconda-extra/).

And then install the packages with ```conda install pytorch torchvision torchaudio pytorch-cuda=11.8 cuda -c pytorch -c nvidia/label/cuda-11.8.0```

## Compiling and Installation

Now we can compile the FSS library and install it so it can be used by the python code.

open a shell, set the working directory to the directory containing this README file, run the setup.py with ```python setup.py```.

If the compiling and installation is successful, then run the example.py with ```python example.py```, it should print ```example test pass``` if the test is successful.

## Some Notes about example.py

The ```cpu_dpf``` and the ```gpu_dpf``` are designed to be interchangeable, which means that you can generate keys with either CPU or GPU, and then evalute the keys with whichever device you like.

Each key passed to the ```rep_eval_wrap``` is evaluated against all the values in ```x```. Put it in another way, ```rep_eval_wrap``` performs ```key_num * x_num``` times evaluation.

The result returned by ```rep_eval_wrap``` is a two-dimensional numpy vectors, with ```rst[i][j]``` being the result of i-th key evaluated on the j-th x.
