use crate::modules::rv32i_alu;
use crate::modules::rv32i_isa;
use crate::modules::utils;
#[allow(dead_code)]
pub struct Rv32iProcessor {
    registers: Vec<u32>,
    pc: u32,
    program: Vec<u32>,
    memory: Vec<u32>,
    state: State,
    isa: rv32i_isa::Rv32iIsa,
    alu: rv32i_alu::Rv32iAlu,
}

enum State {
    Fetch,
    Execute,
    WaitInstr,
    WaitData,
}

#[allow(dead_code)]
impl Rv32iProcessor {
    fn new(program: Vec<u32>, memory: Vec<u32>) -> Self {
        Self {
            registers: vec![0; 32], // Initialize all registers to 0
            program,
            memory,
            pc: 0,
            state: State::Fetch,
            isa: rv32i_isa::Rv32iIsa::new(0),
            alu: rv32i_alu::Rv32iAlu::new(),
        }
    }

    fn exec(&mut self) {
        match self.state {
            State::Fetch => {
                self.isa.i_instruction = self.program[self.pc as usize];

                self.state = State::Execute;
            }
            State::Execute => {
                // Execute instruction
                self.state = State::WaitInstr;
            }
            State::WaitInstr => {
                // Wait for instruction to complete
                self.state = State::WaitData;
            }
            State::WaitData => {
                // Wait for data to complete
                self.state = State::Fetch;
            }
        }
    }
}
