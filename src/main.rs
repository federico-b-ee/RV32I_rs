use rv32i_rs::modules::rv32i_processor::Rv32iProcessor;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cpu = Rv32iProcessor::new_from_elf("example/riscv_asm.elf")?;

    let n_instr = cpu.program.len();
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
    Ok(())
}
