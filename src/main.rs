use rv32i_rs::modules::rv32i_processor::Rv32iProcessor;
use std::fs;

fn main() {
    let contents = fs::read_to_string("example/riscv_asm.prog").expect("Failed to read file");

    let program: Vec<u32> = contents
        .lines()
        .map(|line| u32::from_str_radix(line.trim(), 16).unwrap())
        .collect();

    let contents = fs::read_to_string("example/riscv_asm.mem").expect("Failed to read file");

    let mem_initial: Vec<u32> = contents
        .lines()
        .map(|line| u32::from_str_radix(line.trim(), 16).unwrap())
        .collect();

    let n_instr = program.len();

    // This part is important, since the linker script used configures the memory as 3 pages of 1024 words each.
    // Being a contiguous memory, we need to allocate a vector with the size of 3 * 1024 words.
    // The program has a length of 2K and the memory initial state has a length of 1K.
    // The sp is initialized to 3072 (3K), and it grows downwards to 2048 (2K).
    let mut memory = vec![0; 1024 * 3];
    memory[2048..2048 + mem_initial.len()].copy_from_slice(&mem_initial);
    let mut cpu = Rv32iProcessor::new(program, memory);

    // Some headroom is needed to run the program.
    // The number of instructions don't take into account
    // the instructions needed to perform a multiplication or a division for example
    for _ in 0..n_instr * 10 {
        cpu.exec();
    }

    // The program calculates the 10th number of the Fibonacci sequence and the factorial of 9.
    // The result is stored in registers x23
    let mut fib = (0, 1);
    let mut result = 0;
    for _ in 0..10 {
        let (a, b) = fib;
        result = {
            let sum = a + b;
            fib = (b, sum);
            sum
        }
    }

    assert_eq!(result, cpu.registers[23]);

    // Then, the program calculates several multiplications.
    // The result is stored in registers x24
    let mut result = 1;
    for i in 1..10 {
        result *= i;
    }

    assert_eq!(result, cpu.registers[24]);

    // Finally, the program loads two global variables and stores them in registers x25 and x26.
    assert_eq!(777u32, cpu.registers[25]);
    assert_eq!(1737u32, cpu.registers[26]);
}
