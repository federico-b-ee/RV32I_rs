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

pub const STATE_LEN: u32 = 3;
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

                let loadstore_addr = loadstore_addr / 4;

                let funct3_bytes = utils::u32_to_bitvec(self.isa.o_funct3 as u32);
                // funct3_bytes[0..=1] == 0b00
                let en_byte = matches!(self.isa.o_funct3, 0x0 | 0x4);
                // funct3_bytes[0..=1] == 0b01
                let en_hw = matches!(self.isa.o_funct3, 0x1 | 0x5);

                let memory = &utils::u32_to_bitvec(self.memory[(loadstore_addr) as usize]);

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

                // Store
                let store_data = if self.isa.o_instrtype == InstrType::StoreStype {
                    let rs2_bytes = utils::u32_to_bitvec(self.registers[self.isa.o_rs2 as usize]);
                    let mut bitvec = Vec::new();
                    bitvec.extend_from_slice(&rs2_bytes[0..=7]);
                    if loadstore_addr_bytes[0] == 1 {
                        bitvec.extend_from_slice(&rs2_bytes[0..=7]);
                    } else {
                        bitvec.extend_from_slice(&rs2_bytes[8..=15]);
                    }
                    if loadstore_addr_bytes[1] == 1 {
                        bitvec.extend_from_slice(&rs2_bytes[0..=7]);
                    } else {
                        bitvec.extend_from_slice(&rs2_bytes[16..=23]);
                    }
                    if loadstore_addr_bytes[0] == 1 {
                        bitvec.extend_from_slice(&rs2_bytes[0..=7]);
                    } else if loadstore_addr_bytes[1] == 1 {
                        bitvec.extend_from_slice(&rs2_bytes[8..=15]);
                    } else {
                        bitvec.extend_from_slice(&rs2_bytes[24..=31]);
                    }
                    utils::bitvec_to_u32(&bitvec)
                } else {
                    0
                };

                let store_mask: u32 = if self.isa.o_instrtype == InstrType::StoreStype {
                    if en_byte {
                        if loadstore_addr_bytes[1] == 1 {
                            if loadstore_addr_bytes[0] == 1 {
                                0xFF00_0000
                            } else {
                                0x00FF_0000
                            }
                        } else if loadstore_addr_bytes[0] == 1 {
                            0x0000_FF00
                        } else {
                            0x000_00FF
                        }
                    } else if en_hw {
                        if loadstore_addr_bytes[1] == 1 {
                            0xFFFF_0000
                        } else {
                            0x0000_FFFF
                        }
                    } else {
                        0xFFFF_FFFF
                    }
                } else {
                    0
                };

                if self.isa.o_instrtype == InstrType::StoreStype {
                    let mut mem = self.memory[loadstore_addr as usize];
                    mem |= store_mask & store_data;
                    self.memory[loadstore_addr as usize] = mem;
                }

                let write_destination_register = match self.isa.o_instrtype {
                    InstrType::JalJtype | InstrType::JalrItype => self.pc.wrapping_add(4),
                    InstrType::LuiUtype => self.isa.o_imm,
                    InstrType::AuipcUtype => self.pc.wrapping_add(self.isa.o_imm),
                    InstrType::LoadItype => load_data,
                    InstrType::Illegal => 0,
                    _ => self.alu.o_out,
                };

                if self.isa.o_rd as usize != 0
                    && InstrType::StoreStype != self.isa.o_instrtype
                    && InstrType::BranchBtype != self.isa.o_instrtype
                {
                    self.registers[self.isa.o_rd as usize] = write_destination_register;
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
    fn test_alu() {
        // addi x1, x0, 5
        // addi x2, x0, 12
        // add x3, x1, x2
        let program = vec![0x00500093, 0x00c00113, 0x002081b3];
        let n_instr = program.len() as u32;
        let memory = vec![0; 1024];
        let mut processor = Rv32iProcessor::new(program, memory);

        for i in 0..n_instr {
            for _ in 0..STATE_LEN {
                processor.exec();
            }
            assert_eq!(processor.pc, (i + 1) * 4);
        }
        assert_eq!(processor.registers[3], 17);
    }

    #[test]
    fn test_load_store() {
        // addi x1, x0, 5
        // addi x2, x0, 10
        // addi x3, x0, 0
        // sb x1, 0(x3)
        // sb x2, 1(x3)
        // lb x4, 0(x3)
        // lb x5, 1(x3)
        // add x6, x4, x5
        let program = vec![
            0x00500093, // addi x1, x0, 5
            0x00a00113, // addi x2, x0, 10
            0x00000193, // addi x3, x0, 0
            0x00118023, // sb x1, 0(x3)
            0x002180a3, // sb x2, 1(x3)
            0x00018203, // lb x4, 0(x3)
            0x00118283, // lb x5, 1(x3)
            0x00520333, // add x6, x4, x5
            0x00118123, // sb x1, 2(x3)
            0x001181a3, // sb x1, 3(x3)
            0x00118223, // sb x1, 4(x3)
            0x0001a383, // lw x7, 0(x3)
        ];
        let n_instr = program.len() as u32;
        let memory = vec![0; 1024];
        let mut processor = Rv32iProcessor::new(program, memory);

        for i in 0..n_instr {
            for _ in 0..STATE_LEN {
                processor.exec();
            }
            assert_eq!(processor.pc, (i + 1) * 4);
        }
        assert_eq!(processor.registers[6], 15);
        assert_eq!(processor.registers[7], 0x0505_0A05);
    }
}
