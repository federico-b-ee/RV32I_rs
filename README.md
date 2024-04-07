<h1 align="center">RV32I_rs 🦀</h1>

<div align="center">

[What?](#what) - [Why \& Who?](#why--who) - [How?](#how)

[Improvements](#improvements) - [Resources](#resources)

</div>

## What?

This is a modular emulator for the RV32I processor architecture, written in Rust. It is strongly inspired by Bruno Levy's [implementation](https://github.com/BrunoLevy/learn-fpga) in Verilog.


## Why & Who?

The goal of this project is to provide a way to emulate RISC-V programs written in Rust using an emulator that is also written in Rust. This project is primarily intended for educational purposes.

## How?

To execute the demo, follow these steps:

1. Before proceeding, ensure that `rust` and `cargo` are installed on your system. Then, add the target and install `cargo-binutils` using the following commands:
```sh
rustup target add riscv32i-unknown-none-elf
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

2. Once the repository is cloned. In the terminal, execute the following command:

```sh
make run_demo
```

The provided example is a `program` targeting `riscv32i-unknown-none-elf`. A linker script is required during the build process to define the memory map of the microarchitecture.

A `Makefile` is included, which generates an `.elf` file located at `example/riscv_asm.elf`, along with several binary files in the `example/binaries` directory. Among these binaries, the `.mem` file houses the `.data` section, while the `.prog` file contains the `.text` section.

This implementation offers flexibility in creating a "cpu" instance. You can either use the path to the `.elf` file or the individual binary files.

## Improvements

The linker script(at `example/riscv_asm/link.x`) generates a `memory map`. In this memory scheme, the processor perceives the "program" (or ROM) and the "memory" (or RAM) as the same physical hardware. However, this is not always the case. In fact, this implementation separates the program and the memory.

The memory map, as defined by the linker script, allocates 2K for the program and 1K for the RAM, resulting in a contiguous 3K in total. Consequently, the memory must be a `Vec<u32>` with 3K positions. However, the actual data is stored starting from `mem[2048..]`.

Improving this to utilize less space would be beneficial.

## Resources

- [Preface - The Embedonomicon](https://docs.rust-embedded.org/embedonomicon/preface.html)
- [GitHub - rust-embedded/cargo-binutils: Cargo subcommands to invoke the LLVM tools shipped with the Rust toolchain](https://github.com/rust-embedded/cargo-binutils)

### Linker Script
- [rv-rust-baremetal/link.ld at main · trmckay/rv-rust-baremetal · GitHub](https://github.com/trmckay/rv-rust-baremetal/blob/main/link.ld)
- [Using LD, the GNU linker - Command Language](https://ftp.gnu.org/old-gnu/Manuals/ld-2.9.1/html_chapter/ld_3.html)
- [Everything You Never Wanted To Know About Linker Script · mcyoung](https://mcyoung.xyz/2021/06/01/linker-script/)


### Useful tools
- [rvcodec.js · RISC-V Instruction Encoder/Decoder](https://luplab.gitlab.io/rvcodecjs/)
- [RISC-V Interpreter](https://www.cs.cornell.edu/courses/cs3410/2019sp/riscv/interpreter/)
