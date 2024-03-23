use crate::modules::utils;

#[derive(Debug, PartialEq)]
pub enum InstrType {
    AluRtype,
    AluItype,
    LoadItype,
    StoreStype,
    BranchBtype,
    JalJtype,
    JalrItype,
    LuiUtype,
    AuipcUtype,
    SystemItype,
    Illegal,
}

pub struct Rv32iIsa {
    pub i_instruction: u32,
    pub o_instrtype: InstrType,
    pub o_imm: u32,
    pub o_rs1: u8,
    pub o_rs2: u8,
    pub o_rd: u8,
    pub o_funct3: u8,
    pub o_funct7: u8,
}

#[allow(dead_code)]
impl Rv32iIsa {
    pub fn new(instruction: u32) -> Rv32iIsa {
        Rv32iIsa {
            i_instruction: instruction,
            o_instrtype: InstrType::Illegal,
            o_imm: 0,
            o_rs1: 0,
            o_rs2: 0,
            o_rd: 0,
            o_funct3: 0,
            o_funct7: 0,
        }
    }
    fn parse_instr(&mut self) {
        let bits_instruction = utils::u32_to_bitvec(self.i_instruction);

        match utils::bitvec_to_u32(&bits_instruction[0..=6]) {
            0b011_0011 => self.o_instrtype = InstrType::AluRtype,
            0b001_0011 => self.o_instrtype = InstrType::AluItype,
            0b000_0011 => self.o_instrtype = InstrType::LoadItype,
            0b010_0011 => self.o_instrtype = InstrType::StoreStype,
            0b110_0011 => self.o_instrtype = InstrType::BranchBtype,
            0b110_1111 => self.o_instrtype = InstrType::JalJtype,
            0b110_0111 => self.o_instrtype = InstrType::JalrItype,
            0b011_0111 => self.o_instrtype = InstrType::LuiUtype,
            0b001_0111 => self.o_instrtype = InstrType::AuipcUtype,
            0b111_0011 => self.o_instrtype = InstrType::SystemItype,
            _ => self.o_instrtype = InstrType::Illegal,
        }

        match self.o_instrtype {
            InstrType::AluItype
            | InstrType::LoadItype
            | InstrType::JalrItype
            | InstrType::SystemItype => self.o_imm = Rv32iIsa::parse_imm_itype(&bits_instruction),
            InstrType::StoreStype => self.o_imm = Rv32iIsa::parse_imm_stype(&bits_instruction),
            InstrType::BranchBtype => self.o_imm = Rv32iIsa::parse_imm_btype(&bits_instruction),
            InstrType::JalJtype => self.o_imm = Rv32iIsa::parse_imm_jtype(&bits_instruction),
            InstrType::LuiUtype | InstrType::AuipcUtype => {
                self.o_imm = Rv32iIsa::parse_imm_utype(&bits_instruction)
            }
            _ => self.o_imm = 0b0000_00000,
        }

        // TODO replace the as u8 to a more robust solution
        self.o_rs1 = utils::bitvec_to_u32(&bits_instruction[15..=19]) as u8;
        self.o_rs2 = utils::bitvec_to_u32(&bits_instruction[20..=24]) as u8;
        self.o_rd = utils::bitvec_to_u32(&bits_instruction[7..=11]) as u8;
        self.o_funct3 = utils::bitvec_to_u32(&bits_instruction[12..=14]) as u8;
        self.o_funct7 = utils::bitvec_to_u32(&bits_instruction[25..=31]) as u8;
    }

    fn parse_imm_itype(bits: &[u8]) -> u32 {
        let mut imm_bitvec = Vec::new();
        imm_bitvec.extend_from_slice(&bits[20..=30]);
        imm_bitvec.extend_from_slice(&[bits[31]; 21]);

        utils::bitvec_to_u32(&imm_bitvec)
    }

    fn parse_imm_stype(bits: &[u8]) -> u32 {
        let mut imm_bitvec = Vec::new();
        imm_bitvec.extend_from_slice(&bits[7..=11]);
        imm_bitvec.extend_from_slice(&bits[25..=30]);
        imm_bitvec.extend_from_slice(&[bits[31]; 21]);

        utils::bitvec_to_u32(&imm_bitvec)
    }

    fn parse_imm_btype(bits: &[u8]) -> u32 {
        let mut imm_bitvec = Vec::new();
        imm_bitvec.push(0u8);
        imm_bitvec.extend_from_slice(&bits[8..=11]);
        imm_bitvec.extend_from_slice(&bits[25..=30]);
        imm_bitvec.push(bits[7]);
        imm_bitvec.extend_from_slice(&[bits[31]; 20]);

        utils::bitvec_to_u32(&imm_bitvec)
    }

    fn parse_imm_jtype(bits: &[u8]) -> u32 {
        let mut imm_bitvec = Vec::new();
        imm_bitvec.push(0u8);
        imm_bitvec.extend_from_slice(&bits[21..=30]);
        imm_bitvec.push(bits[20]);
        imm_bitvec.extend_from_slice(&bits[12..=19]);
        imm_bitvec.extend_from_slice(&[bits[31]; 12]);

        utils::bitvec_to_u32(&imm_bitvec)
    }

    fn parse_imm_utype(bits: &[u8]) -> u32 {
        let mut imm_bitvec = Vec::new();
        imm_bitvec.extend_from_slice(&[0u8; 12]);
        imm_bitvec.extend_from_slice(&bits[12..=31]);

        utils::bitvec_to_u32(&imm_bitvec)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_imm_itype() {
        // addi x5, x2, 125
        let instr: u32 = 0x07d10293;
        let bits = utils::u32_to_bitvec(instr);
        assert_eq!(Rv32iIsa::parse_imm_itype(&bits), 125u32);
    }

    #[test]
    fn test_parse_imm_stype() {
        // sw x5, 88(x2)
        let instr: u32 = 0x04512c23;
        let bits = utils::u32_to_bitvec(instr);
        assert_eq!(Rv32iIsa::parse_imm_stype(&bits), 88u32);
    }

    #[test]
    fn test_parse_imm_btype() {
        // beq x5, x2, 74
        let instr: u32 = 0x04228563;
        let bits = utils::u32_to_bitvec(instr);
        assert_eq!(Rv32iIsa::parse_imm_btype(&bits), 74u32);
    }

    #[test]
    fn test_parse_imm_jtype() {
        // jal x8, 44
        let instr: u32 = 0x02c0046f;
        let bits = utils::u32_to_bitvec(instr);
        assert_eq!(Rv32iIsa::parse_imm_jtype(&bits), 44u32);
    }

    #[test]
    fn test_parse_imm_utype() {
        // lui x8, 1339
        // 1339 is the [31:12] immediate value
        let instr: u32 = 0x0053b437;
        let bits = utils::u32_to_bitvec(instr);

        let mut imm_bitvec = Vec::new();
        imm_bitvec.extend_from_slice(&bits[12..=31]);
        imm_bitvec.extend_from_slice(&[0u8; 12]);

        assert_eq!(
            Rv32iIsa::parse_imm_utype(&bits),
            0b00000000_01010011_10110000_00000000u32
        );
    }

    #[test]
    fn test_integration_test1() {
        // sub x5, x1, x2
        let mut isa = Rv32iIsa::new(0x402082b3);

        isa.parse_instr();
        assert_eq!(isa.o_instrtype, InstrType::AluRtype);
        assert_eq!(isa.o_rs1, 1u8);
        assert_eq!(isa.o_rs2, 2u8);
        assert_eq!(isa.o_rd, 5u8);
        assert_eq!(isa.o_funct3, 0x0);
        assert_eq!(isa.o_funct7, 0x20);
    }
    #[test]
    fn test_integration_test2() {
        // beq x5, x2, 74
        let mut isa = Rv32iIsa::new(0x04228563);

        isa.parse_instr();
        assert_eq!(isa.o_instrtype, InstrType::BranchBtype);
        assert_eq!(isa.o_imm, 74u32);
        assert_eq!(isa.o_rs1, 5u8);
        assert_eq!(isa.o_rs2, 2u8);
        assert_eq!(isa.o_funct3, 0x0);
    }
}
