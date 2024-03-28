use crate::modules::rv32i_alu;
use crate::modules::rv32i_isa;
use crate::modules::utils;

use super::rv32i_isa::InstrType;
#[allow(dead_code)]
#[derive(Default)]
pub struct Rv32iProcessor {
    registers: Vec<u32>,
    pc: u32,
    program: Vec<u32>,
    memory: Vec<u32>,
    state: State,
    isa: rv32i_isa::Rv32iIsa,
    alu: rv32i_alu::Rv32iAlu,
}

#[derive(Default)]
enum State {
    #[default]
    Fetch,
    Execute,
    EndInstr,
}

#[allow(dead_code)]
impl Rv32iProcessor {
    fn new(program: Vec<u32>, memory: Vec<u32>) -> Self {
        Self {
            registers: vec![0; 32], // Initialize all registers to 0
            program,
            memory,
            pc: 0,
            ..Default::default()
        }
    }

    fn exec(&mut self) {
        match self.state {
            State::Fetch => {
                // This div_ceil is a function that divides the pc by 4 and rounds up
                // May cause bugs if pc is not a multiple of 4
                self.isa.i_instruction = self.program[(self.pc.div_ceil(4)) as usize];
                self.isa.parse_instr();

                self.state = State::Execute;
            }
            State::Execute => {
                let in2 = if self.isa.o_instrtype == rv32i_isa::InstrType::AluRtype
                    || self.isa.o_instrtype == rv32i_isa::InstrType::BranchBtype
                {
                    self.registers[self.isa.o_rs2 as usize]
                } else {
                    self.isa.o_imm
                };

                self.alu.exec(
                    self.registers[self.isa.o_rs1 as usize],
                    in2,
                    self.isa.o_funct3,
                    self.isa.o_funct7,
                    self.isa.i_instruction,
                );
                self.state = State::EndInstr;

                let takebranch = match self.isa.o_funct3 {
                    0x0 => self.alu.o_eq,
                    0x1 => !self.alu.o_eq,
                    0x4 => self.alu.o_lt,
                    0x5 => !self.alu.o_lt,
                    0x6 => self.alu.o_ltu,
                    0x7 => !self.alu.o_ltu,
                    _ => false,
                };

                if InstrType::BranchBtype == self.isa.o_instrtype && takebranch
                    || InstrType::JalJtype == self.isa.o_instrtype
                {
                    self.pc = self.pc.wrapping_add(self.isa.o_imm);
                } else if InstrType::JalrItype == self.isa.o_instrtype {
                    self.pc = self.alu.o_alu_add;
                } else {
                    self.pc = self.pc.wrapping_add(4u32);
                }
            }
            State::EndInstr => {
                let loadstore_addr =
                    self.registers[self.isa.o_rs1 as usize].wrapping_add(self.isa.o_imm);

                let loadstore_addr_bytes = utils::u32_to_bitvec(loadstore_addr);

                let funct3_bytes = utils::u32_to_bitvec(self.isa.o_funct3 as u32);
                // funct3_bytes[0..=1] == 0b00
                let en_byte = matches!(self.isa.o_funct3, 0x0 | 0x4);
                // funct3_bytes[0..=1] == 0b01
                let en_hw = matches!(self.isa.o_funct3, 0x1 | 0x5);

                // placeholder for memory
                let memory = &utils::u32_to_bitvec(2);

                let load_h = if loadstore_addr_bytes[1] == 1 {
                    &memory[16..=31]
                } else {
                    &memory[0..=15]
                };
                let load_b = if loadstore_addr_bytes[0] == 1 {
                    &load_h[8..=15]
                } else {
                    &memory[0..=7]
                };

                let load_sign = funct3_bytes[1] == 0
                    && (if en_byte {
                        load_b[7] == 1
                    } else {
                        load_h[15] == 1
                    });

                let load_data = if en_byte {
                    let mut bitvec = Vec::new();
                    bitvec.extend_from_slice(load_b);
                    bitvec.extend_from_slice(&[load_sign as u8; 24]);

                    utils::bitvec_to_u32(&bitvec)
                } else if en_hw {
                    let mut bitvec = Vec::new();
                    bitvec.extend_from_slice(load_h);
                    bitvec.extend_from_slice(&[load_sign as u8; 16]);

                    utils::bitvec_to_u32(&bitvec)
                } else {
                    utils::bitvec_to_u32(&memory[0..=31])
                };

                let write_data = match self.isa.o_instrtype {
                    InstrType::JalJtype | InstrType::JalrItype => self.pc.wrapping_add(4),
                    InstrType::LuiUtype => self.isa.o_imm,
                    InstrType::AuipcUtype => self.pc.wrapping_add(self.isa.o_imm),
                    InstrType::LoadItype => load_data,
                    InstrType::Illegal => 0,
                    _ => self.alu.o_out,
                };

                if self.isa.o_rd as usize != 0
                    && self.isa.o_instrtype != InstrType::StoreStype
                    && self.isa.o_instrtype != InstrType::BranchBtype
                    && self.isa.o_instrtype != InstrType::LoadItype
                {
                    self.registers[self.isa.o_rd as usize] = write_data;
                }
                self.state = State::Fetch;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec() {
        // addi x1, x0, 5
        // addi x2, x0, 12
        // add x3, x1, x2
        let program = vec![0x00500093, 0x00c00113, 0x002081b3];
        let n_instr = program.len() as u32;
        let memory = vec![0x00000000];
        let mut processor = Rv32iProcessor::new(program, memory);

        for i in 0..n_instr {
            for _ in 0..4 {
                processor.exec();
            }
            assert_eq!(processor.pc, (i + 1) * 4);
        }
        print!("Registers: {:?}", processor.registers);
        assert_eq!(processor.registers[3], 17);
        println!("pc in hex: {:x}", processor.pc);
        println!("pc in hex: 0X{:08X}", processor.pc);
    }
}
