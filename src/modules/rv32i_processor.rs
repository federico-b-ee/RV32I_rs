use crate::modules::rv32i_alu;
use crate::modules::rv32i_isa;
use crate::modules::utils;

use super::rv32i_isa::InstrType;

use object::{Object, ObjectSection};
use std::fs;
#[allow(dead_code)]
#[derive(Default)]
pub struct Rv32iProcessor {
    pub registers: Vec<u32>,
    pub pc: u32,
    pub program: Vec<u32>,
    pub memory: Vec<u32>,
    pub isa: rv32i_isa::Rv32iIsa,
    pub alu: rv32i_alu::Rv32iAlu,
}

#[allow(dead_code)]
impl Rv32iProcessor {
    pub fn new(program: Vec<u32>, memory: Vec<u32>) -> Self {
        Self {
            registers: vec![0; 32], // Initialize all registers to 0
            program,
            memory,
            pc: 0,
            ..Default::default()
        }
    }

    pub fn new_from_elf(elf_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let binary_data = fs::read(elf_path)?;
        let file = object::File::parse(&*binary_data)?;
        let mut data_section: &[u8] = &[0u8];
        let mut text_section: &[u8] = &[0u8];
        for section in file.sections() {
            match section.name().unwrap() {
                ".text" => {
                    text_section = section.data()?;
                }
                ".data" => {
                    data_section = section.data()?;
                }
                _ => {}
            }
        }
        let mut memory_elf = Vec::new();
        for i in (0..data_section.len()).step_by(4) {
            let word = u32::from_le_bytes([
                data_section[i],
                data_section[i + 1],
                data_section[i + 2],
                data_section[i + 3],
            ]);
            memory_elf.push(word);
        }
        let mut program_elf = Vec::new();
        for i in (0..text_section.len()).step_by(4) {
            let word = u32::from_le_bytes([
                text_section[i],
                text_section[i + 1],
                text_section[i + 2],
                text_section[i + 3],
            ]);
            program_elf.push(word);
        }

        // This part is important, since the linker script used configures the memory as 3 pages of 1024 words each.
        // Being a contiguous memory, we need to allocate a vector with the size of 3 * 1024 words.
        // The program has a length of 2K and the memory initial state has a length of 1K.
        // The sp is initialized to 3072 (3K), and it grows downwards to 2048 (2K).
        let mut memory_map = vec![0; 1024 * 3];
        memory_map[2048..2048 + memory_elf.len()].copy_from_slice(&memory_elf);
        memory_map[..program_elf.len()].copy_from_slice(&program_elf);

        Ok(Self {
            registers: vec![0; 32],
            pc: 0,
            program: memory_map[..2048].to_vec(),
            memory: memory_map,
            ..Default::default()
        })
    }

    pub fn exec(&mut self) {
        //State::Fetch => {
        // This div_ceil is a function that divides the pc by 4 and rounds up
        // May cause bugs if pc is not a multiple of 4
        self.isa.i_instruction = self.program[(self.pc.div_ceil(4)) as usize];
        self.isa.parse_instr();

        //State::Execute => {
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

        //State::EndInstr => {
        let load_data = if InstrType::LoadItype == self.isa.o_instrtype
            || InstrType::StoreStype == self.isa.o_instrtype
        {
            let loadstore_addr =
                self.registers[self.isa.o_rs1 as usize].wrapping_add(self.isa.o_imm);

            let loadstore_addr_bytes = utils::u32_to_bitvec(loadstore_addr);

            // This ommits the last 2 bits of the address
            // with verilog: wire [29:0] ADDR = i_addr[31:2];
            let loadstore_addr = loadstore_addr & !0x3;

            let funct3_bytes = utils::u32_to_bitvec(self.isa.o_funct3 as u32);
            // funct3_bytes[0..=1] == 0b00
            let en_byte = matches!(self.isa.o_funct3, 0x0 | 0x4);
            // funct3_bytes[0..=1] == 0b01
            let en_hw = matches!(self.isa.o_funct3, 0x1 | 0x5);

            let memory = &utils::u32_to_bitvec(self.memory[loadstore_addr as usize]);

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
                mem &= !store_mask;
                mem |= store_mask & store_data;
                self.memory[loadstore_addr as usize] = mem;
            }

            load_data
        } else {
            0
        };

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

        let takebranch = match self.isa.o_funct3 {
            0x0 => self.alu.o_eq,
            0x1 => !self.alu.o_eq,
            0x4 => self.alu.o_lt,
            0x5 => !self.alu.o_lt,
            0x6 => self.alu.o_ltu,
            0x7 => !self.alu.o_ltu,
            _ => false,
        };

        if (InstrType::BranchBtype == self.isa.o_instrtype && takebranch)
            || InstrType::JalJtype == self.isa.o_instrtype
        {
            self.pc = self.pc.wrapping_add(self.isa.o_imm);
        } else if InstrType::JalrItype == self.isa.o_instrtype {
            let bitvec = utils::u32_to_bitvec(self.alu.o_alu_add);
            let mut temp_bitvec = Vec::new();
            temp_bitvec.push(0u8);
            temp_bitvec.extend_from_slice(&bitvec[1..=31]);
            self.pc = utils::bitvec_to_u32(&temp_bitvec);
        } else {
            self.pc = self.pc.wrapping_add(4u32);
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
            processor.exec();
            assert_eq!(processor.pc, (i + 1) * 4);
        }
        assert_eq!(processor.registers[3], 17);
    }

    #[test]
    fn test_load_store() {
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
            0x0011a223, // sw x1, 4(x3)
            0x0001a383, // lw x7, 0(x3)
            0x0041a403, // lw x8, 4(x3)
        ];
        let n_instr = program.len() as u32;
        let memory = vec![0; 1024];
        let mut processor = Rv32iProcessor::new(program, memory);

        for i in 0..n_instr {
            processor.exec();

            assert_eq!(processor.pc, (i + 1) * 4);
        }
        assert_eq!(processor.registers[6], 15);
        assert_eq!(processor.registers[7], 0x0505_0A05);
        assert_eq!(processor.registers[8], 5);
    }

    #[test]
    fn test_branch() {
        let program = vec![
            0x00f00193, // addi x3, x0, 15
            0x00108093, // addi x1, x1, 1
            0xfe309ee3, // bne x1, x3, -4
        ];
        let n_instr = program.len() as u32;
        let memory = vec![0; 1024];
        let mut processor = Rv32iProcessor::new(program, memory);

        for _ in 0..n_instr * 10 {
            processor.exec();
        }
        println!("{:X}", processor.pc);
        assert_eq!(processor.registers[1], 15); // Check that counter reached 15
    }

    #[test]
    fn test_jalr() {
        // jalr x5, 12(x0)
        let program = vec![0x00c002e7];
        let n_instr = program.len() as u32;
        let memory = vec![0; 1024];
        let mut processor = Rv32iProcessor::new(program, memory);

        for _ in 0..n_instr {
            processor.exec();
        }
        assert_eq!(12, processor.pc);
        assert_eq!(processor.registers[5], 4);
    }

    #[test]
    fn test_jal() {
        // jal x1, 12
        let program = vec![0x00c000ef];
        let n_instr = program.len() as u32;
        let memory = vec![0; 1024];
        let mut processor = Rv32iProcessor::new(program, memory);

        for _ in 0..n_instr {
            processor.exec();
        }
        assert_eq!(processor.registers[1], 4);
    }
}
