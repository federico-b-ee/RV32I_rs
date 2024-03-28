use crate::modules::utils;
pub struct Rv32iAlu {
    pub i_in1: u32,
    pub i_in2: u32,
    pub o_out: u32,
    pub o_eq: bool,
    pub o_lt: bool,
    pub o_ltu: bool,
    pub o_alu_add: u32,
}

impl Default for Rv32iAlu {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl Rv32iAlu {
    pub fn new() -> Rv32iAlu {
        Rv32iAlu {
            i_in1: 0,
            i_in2: 0,
            o_out: 0,
            o_eq: false,
            o_lt: false,
            o_ltu: false,
            o_alu_add: 0,
        }
    }

    pub fn exec(&mut self, in1: u32, in2: u32, funct3: u8, funct7: u8, instr: u32) {
        let instr_bits = utils::u32_to_bitvec(instr);

        self.i_in1 = in1;
        self.i_in2 = in2;
        let shamt = if instr_bits[5] == 1 {
            utils::bitvec_to_u32(&(utils::u32_to_bitvec(self.i_in2)[0..=4]))
        } else {
            utils::bitvec_to_u32(&instr_bits[20..=24])
        };

        let alu_add = self.i_in1.wrapping_add(self.i_in2);
        let alu_sub = self.i_in1.wrapping_sub(self.i_in2);

        let mut zeroes = vec![0; 31];
        match funct3 {
            0x0 => self.o_out = if funct7 == 0x20 { alu_sub } else { alu_add },
            0x1 => self.o_out = self.i_in1 << shamt,
            0x2 => {
                self.o_out = {
                    zeroes.push(self.o_lt as u8);
                    utils::bitvec_to_u32(&zeroes)
                }
            }
            0x3 => {
                self.o_out = {
                    zeroes.push(self.o_ltu as u8);
                    utils::bitvec_to_u32(&zeroes)
                }
            }
            0x4 => self.o_out = self.i_in1 ^ self.i_in2,
            0x5 => {
                self.o_out = {
                    if funct7 == 0x20 {
                        // arithmetic shift right
                        (self.i_in1 as i32 >> shamt) as u32
                    } else {
                        // logical shift right
                        self.i_in1 >> shamt
                    }
                }
            }
            0x6 => self.o_out = self.i_in1 | self.i_in2,
            0x7 => self.o_out = self.i_in1 & self.i_in2,
            _ => self.o_out = 0,
        }

        self.o_eq = self.i_in1 == self.i_in2;
        self.o_lt = (self.i_in1 as i32) < (self.i_in2 as i32);
        self.o_ltu = self.i_in1 < self.i_in2;
        self.o_alu_add = alu_add;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift() {
        let mut alu = Rv32iAlu::new();

        // Test case 1
        // srai x1, x2, 4
        // arithmetic shift right
        let in1 = 0xFFFF_0000;
        alu.exec(in1, 3, 0x5, 0x20, 0x40415093);
        assert_eq!(alu.o_out, ((in1 as i32) >> 4) as u32);

        // Test case 2
        // srli x1, x2, 4
        // logical shift right
        alu.exec(in1, 2, 0x5, 0x00, 0x00415093);
        assert_eq!(alu.o_out, in1 >> 4);
    }

    #[test]
    fn test_add() {
        let mut alu = Rv32iAlu::new();

        // Test case 1
        // add x3, x1, x2
        let in1: u32 = 0xFFFF_FFF4;
        let in2: u32 = 0x0000_000F;
        // 0xFFFF_FFF4 + 0x0000_000F = 0x0000_0003
        // in1.wrapping_add(in2) = 0x0000_0003
        alu.exec(in1, in2, 0x0, 0x00, 0x0000_0000);
        assert_eq!(alu.o_out, 3u32);

        // Test case 2
        // addi x1, x2, 125
        let in1 = 50;
        let in2 = -125i32 as u32;
        alu.exec(in1, in2, 0x0, 0x00, 0x0000_0000);
        assert_eq!(alu.o_out, -75i32 as u32);
    }

    #[test]
    fn test_lt_ltu() {
        let mut alu = Rv32iAlu::new();

        alu.exec(2, 4, 0x0, 0x00, 0x0000_0000);
        assert!(alu.o_lt);

        alu.exec((-2i32) as u32, 4, 0x0, 0x00, 0x0000_0000);
        // -2 > 4 -> false
        assert!(!alu.o_ltu);

        alu.exec((-2i32) as u32, 4, 0x0, 0x00, 0x0000_0000);
        assert!(alu.o_lt);
    }
}
