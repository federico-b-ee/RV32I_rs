use rv32i_rs::modules::utils;

fn main() {
    let x: u32 = 3;
    let bits = utils::u32_to_bitvec(x);

    match utils::bitvec_to_u32(&bits[0..=4]) {
        0b00 => println!("Zero"),
        0b01 => println!("One"),
        0b10 => println!("Two"),
        0b11 => println!("Three"),
        _ => println!("Not zero"),
    }
}
