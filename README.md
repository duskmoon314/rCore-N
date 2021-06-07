# rCore-N

> submodule of [Gallium70/rv-n-ext-impl](https://github.com/Gallium70/rv-n-ext-impl)

Based on [duskmoon314/rCore:ch6](https://github.com/duskmoon314/rCore/tree/ch6) (which is based on [rcore-os/rCore-Tutorial-v3:ch6](https://github.com/rcore-os/rCore-Tutorial-v3/tree/ch6))

Add support of **user mode trap**

## How to run

I prefer to use `just` and do not guarantee the correctness of the `makefile` in os.

### Build the environment

#### QEMU with extN support

I have added N Extension CSRs of RISC-V support to stable-5.0 of QEMU. [duskmoon314/qemu:riscv-N-stable-5.0](https://github.com/duskmoon314/qemu)

```bash
git clone https://github.com/duskmoon314/qemu.git
mkdir qemu-build
cd qemu-build
../qemu/configure --target-list="riscv64-softmmu"
make -j8
```

#### install just

We can install just via cargo

```bash
cargo install just
```

### build and run

```bash
cd os
LOG=DEBUG just run
```

> There are 5 levels for logger
>
> ERROR, WARN, INFO, DEBUG, TRACE
>
> Use via `LOG=XXXXX just run`
