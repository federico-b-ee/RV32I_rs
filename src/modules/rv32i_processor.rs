#[allow(dead_code)]
struct Rv32iProcessor {
    registers: Vec<u32>,
    pc: u32,
    instr: u32,
    program: Vec<u32>,
    memory: Vec<u32>,
    state: State,
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
            instr: 0,
            state: State::Fetch,
        }
    }

    fn exec(&mut self) {
        match self.state {
            State::Fetch => {
                self.instr = self.program[self.pc as usize];
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
