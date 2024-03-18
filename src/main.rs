use rv32i_rs::modules::utils;

fn main() {
    // -3
    let x = 0xFFFF_0000 as u32;
    let shamt = 4;

    let c = x >> shamt;
    let d = (x as i32 >> shamt) as u32;
    println!("c: {:?}, d: {:?}", c, d);
}
