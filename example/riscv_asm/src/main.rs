#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::global_asm;

global_asm!(include_str!("asm/init.S"));

use core::panic::PanicInfo;
pub extern "C" fn main() -> u32 {
    // The .bss section is used for uninitialized data, so you can't have non-zero initializers in this section.
    let global: u32 = 777;
    // This needs to be in the .data section, which is used for initialized data.
    // without this attribute, the linker will not be able to find the variable.
    #[link_section = ".data"]
    static GLOBAL: u32 = 1737;
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
    unsafe {
        asm!("add x23, x0, {}", in(reg) result);
    }
    let mut result = 1;
    for i in 1..10 {
        result *= i;
    }
    unsafe {
        asm!("add x24, x0, {}", in(reg) result);
    }
    unsafe {
        asm!("add x25, x0, {}", in(reg) global);
        asm!("add x26, x0, {}", in(reg) GLOBAL);
    }
    0
}

#[no_mangle]
pub extern "C" fn _rust_entry() -> ! {
    unsafe {
        asm!("addi x27, x0, 27");
    }
    main();
    panic!();
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}
