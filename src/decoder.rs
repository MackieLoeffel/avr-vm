use data::{Instruction, LDType, LDMode, LPMType, X, Y, Z};
use data::Instruction::*;
use data::SetClear::*;
use util::{bits, bits16};

pub fn decode<T>(asm: T) -> DecodedIterator<T> where T: Iterator<Item=u8> { DecodedIterator {iter: asm, next_invalid: false} }

pub struct DecodedIterator<T: Iterator<Item=u8>> { iter: T, next_invalid: bool }

impl<T: Iterator<Item=u8>> Iterator for DecodedIterator<T> {
    type Item=Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_invalid {
            self.next_invalid = false;
             // not the correct behavior, when somebody jumps in the
             // middle of a instruction, but a program shouldn't do it anyway
            return Some(SecondOpWord);
        }

        let b = match self.read_u16() {
            Ok(byte) => byte,
            Err(opt) => match opt {
                None => return None,
                Some(byte) => return Some(IncompleteOp(byte as u32))
            }
        };

        // decoding happens here
        let ret = match bits(b, 12, 4) {
            0b0000 =>
                match bits(b, 10, 2) {
                    0b00 =>
                        match bits(b, 8, 2) {
                            0b00 => match bits(b, 0, 8) {
                                0 => NOP,
                                _ => UnknownOp(b)
                            },
                            0b01 => MOVW(bits(b, 4, 4) << 1, bits(b, 0, 4) << 1),
                            _ => UnknownOp(b)
                        },
                    0b01 => CPC(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b10 => SBC(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b11 => ADD(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    _ => UnknownOp(b)
                },
            0b0001 =>
                match bits(b, 10, 2) {
                    0b00 => CPSE(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b01 => CP(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b10 => SUB(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b11 => ADC(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    _ => UnknownOp(b)
                },
            0b0010 =>
                match bits(b, 10, 2) {
                    0b00 => AND(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b01 => EOR(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b10 => OR(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    0b11 => MOV(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    _ => UnknownOp(b)
                },
            0b0011 => CPI(bits(b, 4, 4) + 16, bits(b, 8, 4) << 4 | bits(b, 0, 4)),
            0b0100 => SBCI(bits(b, 4, 4) + 16, bits(b, 8, 4) << 4 | bits(b, 0, 4)),
            0b0101 => SUBI(bits(b, 4, 4) + 16, bits(b, 8, 4) << 4 | bits(b, 0, 4)),
            0b0111 => ANDI(bits(b, 4, 4) + 16, bits(b, 8, 4) << 4 | bits(b, 0, 4)),
            0b0110 => ORI(bits(b, 4, 4) + 16, bits(b, 8, 4) << 4 | bits(b, 0, 4)),
            0b1000 =>
                LD_ST(if bits(b, 9, 1) == 1 {LDType::ST} else {LDType::LD},
                    bits(b, 4, 5),
                    if bits(b, 3, 1) == 1 {Y} else {Z},
                    LDMode::Displacement(bits(b, 10, 2) << 3 | bits(b, 0, 3))),
            0b1001 =>
                match bits(b, 10, 2) {
                    0b00 =>
                        match bits(b, 9, 1) {
                            0 =>
                                match bits(b, 0, 4) {
                                    0b0000 => match self.read_second_part(b) {
                                        Ok(b2) => LD_STS(LDType::LD, bits(b, 4, 5), b2),
                                        Err(i) => i
                                    },
                                    0b0001 => LD_ST(LDType::LD, bits(b, 4, 5), Z, LDMode::PostIncrement),
                                    0b0010 => LD_ST(LDType::LD, bits(b, 4, 5), Z, LDMode::PreDecrement),
                                    0b0100 => LPM(bits(b, 4, 5), LPMType::Z),
                                    0b0101 => LPM(bits(b, 4, 5), LPMType::ZPostIncrement),
                                    0b1001 => LD_ST(LDType::LD, bits(b, 4, 5), Y, LDMode::PostIncrement),
                                    0b1010 => LD_ST(LDType::LD, bits(b, 4, 5), Y, LDMode::PreDecrement),
                                    0b1100 => LD_ST(LDType::LD, bits(b, 4, 5), X, LDMode::Displacement(0)),
                                    0b1101 => LD_ST(LDType::LD, bits(b, 4, 5), X, LDMode::PostIncrement),
                                    0b1110 => LD_ST(LDType::LD, bits(b, 4, 5), X, LDMode::PreDecrement),
                                    0b1111 => POP(bits(b, 4, 5)),
                                    _ => UnknownOp(b)
                                },
                            1 =>
                                match bits(b, 0, 4) {
                                    0b0000 => match self.read_second_part(b) {
                                        Ok(b2) => LD_STS(LDType::ST, bits(b, 4, 5), b2),
                                        Err(i) => i
                                    },
                                    0b0001 => LD_ST(LDType::ST, bits(b, 4, 5), Z, LDMode::PostIncrement),
                                    0b0010 => LD_ST(LDType::ST, bits(b, 4, 5), Z, LDMode::PreDecrement),
                                    // 0b0101 => LAS(bits(b, 4, 5)),
                                    // 0b0110 => LAC(bits(b, 4, 5)),
                                    // 0b0111 => LAT(bits(b, 4, 5)),
                                    0b1001 => LD_ST(LDType::ST, bits(b, 4, 5), Y, LDMode::PostIncrement),
                                    0b1010 => LD_ST(LDType::ST, bits(b, 4, 5), Y, LDMode::PreDecrement),
                                    0b1100 => LD_ST(LDType::ST, bits(b, 4, 5), X, LDMode::Displacement(0)),
                                    0b1101 => LD_ST(LDType::ST, bits(b, 4, 5), X, LDMode::PostIncrement),
                                    0b1110 => LD_ST(LDType::ST, bits(b, 4, 5), X, LDMode::PreDecrement),
                                    0b1111 => PUSH(bits(b, 4, 5)),
                                    _ => UnknownOp(b)
                                },
                            _ => UnknownOp(b)
                        },
                    0b01 =>
                        match bits(b, 9, 1) {
                            0 =>
                                match bits(b, 1, 3) {
                                    0b000 => match bits(b, 0, 1) {
                                        0 => COM(bits(b, 4, 5)),
                                        1 => NEG(bits(b, 4, 5)),
                                        _ => UnknownOp(b)
                                    },
                                    0b001 => match bits(b, 0, 1) {
                                        0 => SWAP(bits(b, 4, 5)),
                                        1 => INC(bits(b, 4, 5)),
                                        _ => UnknownOp(b)
                                    },
                                    0b010 => match bits(b, 0, 1) {
                                        1 => ASR(bits(b, 4, 5)),
                                        _ => UnknownOp(b)
                                    },
                                    0b011 => match bits(b, 0, 1) {
                                        0 => LSR(bits(b, 4, 5)),
                                        1 => ROR(bits(b, 4, 5)),
                                        _ => UnknownOp(b)
                                    },
                                    0b100 => match bits(b, 0, 1) {
                                        0 =>
                                            match bits(b, 8, 1) {
                                                0 => (if bits(b, 7, 1) == 0 {BSET} else {BCLR})(bits(b, 4, 3)),
                                                1 => match bits(b, 4, 4) {
                                                    0b0000 => RET,
                                                    0b0001 => RETI,
                                                    0b1000 => SLEEP,
                                                    0b1100 => LPM(0, LPMType::Z),
                                                    _ => UnknownOp(b)
                                                },
                                                _ => UnknownOp(b)
                                            },
                                        1 => match bits(b, 4, 5) {
                                            0b10000 => ICALL,
                                            _ => UnknownOp(b)
                                        },
                                        _ => UnknownOp(b)
                                    },
                                    0b101 => match bits(b, 0, 1) {
                                        0 => DEC(bits(b, 4, 5)),
                                        _ => UnknownOp(b)
                                    },
                                    0b110 =>
                                        match self.read_second_part(b) {
                                            Ok(b2) => JMP(
                                                ((bits(b, 4, 5) << 1 | bits(b, 0, 1)) as u32) << 16 | (b2 as u32)),
                                            Err(i) => i
                                        },
                                    0b111 =>
                                        match self.read_second_part(b) {
                                            Ok(b2) => CALL(
                                                ((bits(b, 4, 5) << 1 | bits(b, 0, 1)) as u32) << 16 | (b2 as u32)),
                                            Err(i) => i
                                        },
                                    _ => UnknownOp(b)
                                },
                            1 => match bits(b, 8, 1) {
                                0 => ADIW((bits(b, 4, 2) + 12) << 1, bits(b, 6, 2) << 4 | bits(b, 0, 4)),
                                1 => SBIW((bits(b, 4, 2) + 12) << 1, bits(b, 6, 2) << 4 | bits(b, 0, 4)),
                                _ => UnknownOp(b)
                            },
                            _ => UnknownOp(b)
                        },
                    0b10 =>
                        match bits(b, 8, 2) {
                            0b00 => C_SBI(Clear, bits(b, 3, 5), bits(b, 0, 3)),
                            0b01 => SBIC_S(Clear, bits(b, 3, 5), bits(b, 0, 3)),
                            0b10 => C_SBI(Set, bits(b, 3, 5), bits(b, 0, 3)),
                            0b11 => SBIC_S(Set, bits(b, 3, 5), bits(b, 0, 3)),
                            _ => UnknownOp(b)
                        },
                    0b11 => MUL(bits(b, 4, 5), bits(b, 9, 1) << 4 | bits(b, 0, 4)),
                    _ => UnknownOp(b)
                },
            0b1010 =>
                LD_ST(if bits(b, 9, 1) == 1 {LDType::ST} else {LDType::LD},
                    bits(b, 4, 5),
                    if bits(b, 3, 1) == 1 {Y} else {Z},
                    LDMode::Displacement(1 << 5 | bits(b, 10, 2) << 3 | bits(b, 0, 3))),
            0b1011 =>
                (if bits(b, 11, 1) == 0 {IN} else {OUT})(bits(b, 4, 5), bits(b, 9, 2) << 4 | bits(b, 0, 4)),
            0b1100 => RJMP(-((bits16(b, 11, 1) as i16) << 11)  + bits16(b, 0, 11) as i16 + 1),
            0b1101 => RCALL(-((bits16(b, 11, 1) as i16) << 11)  + bits16(b, 0, 11) as i16 + 1),
            0b1110 => LDI(bits(b, 4, 4) + 16, bits(b, 8, 4) << 4 | bits(b, 0, 4)),
            0b1111 =>
                match bits(b, 11, 1) {
                    0b0 => {
                        let k =  -((bits(b, 9, 1) as i8) << 6) + (bits(b, 3, 6) as i8)
                            // we add the +1 here for simplification,
                            // which should be added during the
                            // command according to the spec
                            + 1;
                        BRBC_S(if bits(b, 10, 1) == 0 {Set} else {Clear}, bits(b,0,3), k)
                    }
                    0b1 =>
                        match bits(b, 3, 1) {
                            0 => match bits(b, 9, 2) {
                                0b00 => BLD_ST(LDType::LD, bits(b, 4, 5), bits(b, 0, 3)),
                                0b01 => BLD_ST(LDType::ST, bits(b, 4, 5), bits(b, 0, 3)),
                                0b10 => SBR(Clear, bits(b, 4, 5), bits(b, 0, 3)),
                                0b11 => SBR(Set, bits(b, 4, 5), bits(b, 0, 3)),
                                _ => UnknownOp(b)
                            },
                            _ => UnknownOp(b)
                        },
                    _ => UnknownOp(b)
                },
            _ => UnknownOp(b)
        };

        Some(ret)
    }
}

impl<T: Iterator<Item=u8>> DecodedIterator<T> {

    #[inline]
    fn read_u16(&mut self) -> Result<u16, Option<u8>> {
        let byte1 = match self.iter.next() {
            Some(byte) => byte,
            None => return Err(None)
        };
        let byte2 = match self.iter.next() {
            Some(byte) => byte,
            None => return Err(Some(byte1))
        };

        let b = (byte1 as u16) | (byte2 as u16) << 8;
        Ok(b)
    }

    fn read_second_part(&mut self, first: u16) -> Result<u16, Instruction> {
        match self.read_u16() {
            Ok(byte) => {
                self.next_invalid = true;
                Ok(byte)
            },
            Err(opt) => match opt {
                None => Err(IncompleteOp(first as u32)),
                Some(byte) => Err(IncompleteOp((first as u32) << 8 | (byte as u32)))
            }
        }
    }
}

// #[cfg(test_off)]
#[cfg(test)]
mod tests {
    use super::*;
    use util::assemble;
    use data::{Instruction, LDMode, LDType, LPMType, X, Y, Z};

    #[test]
    fn add() {
        decode_expect("ADD R1, R1", vec![ADD(1, 1)]);
        decode_expect("ADD R2, R2", vec![ADD(2, 2)]);
        decode_expect("ADD R4, R4", vec![ADD(4, 4)]);
        decode_expect("ADD R8, R8", vec![ADD(8, 8)]);
        decode_expect("ADD R16, R16", vec![ADD(16, 16)]);
        decode_expect("ADD R3, R5", vec![ADD(3, 5)]);
        decode_expect("ADD R31, R0", vec![ADD(31, 0)]);
        decode_expect("ADD R0, R31", vec![ADD(0, 31)]);
        decode_expect("ADD R17, R17", vec![ADD(17, 17)]);

        decode_expect("LSL R0", vec![ADD(0, 0)]);
        decode_expect("LSL R1", vec![ADD(1, 1)]);
        decode_expect("LSL R2", vec![ADD(2, 2)]);
        decode_expect("LSL R4", vec![ADD(4, 4)]);
        decode_expect("LSL R8", vec![ADD(8, 8)]);
        decode_expect("LSL R16", vec![ADD(16, 16)]);
        decode_expect("LSL R31", vec![ADD(31, 31)]);
    }

    #[test]
    fn adc() {
        decode_expect("ADC R1, R1", vec![ADC(1, 1)]);
        decode_expect("ADC R2, R2", vec![ADC(2, 2)]);
        decode_expect("ADC R4, R4", vec![ADC(4, 4)]);
        decode_expect("ADC R8, R8", vec![ADC(8, 8)]);
        decode_expect("ADC R16, R16", vec![ADC(16, 16)]);
        decode_expect("ADC R3, R5", vec![ADC(3, 5)]);
        decode_expect("ADC R31, R0", vec![ADC(31, 0)]);
        decode_expect("ADC R0, R31", vec![ADC(0, 31)]);
        decode_expect("ADC R17, R17", vec![ADC(17, 17)]);
    }

    #[test]
    fn adiw() {
        decode_expect("ADIW R24, 0", vec![ADIW(24, 0)]);
        decode_expect("ADIW R26, 1", vec![ADIW(26, 1)]);
        decode_expect("ADIW R28, 2", vec![ADIW(28, 2)]);
        decode_expect("ADIW R30, 4", vec![ADIW(30, 4)]);
        decode_expect("ADIW R26, 8", vec![ADIW(26, 8)]);
        decode_expect("ADIW R26, 16", vec![ADIW(26, 16)]);
        decode_expect("ADIW R26, 32", vec![ADIW(26, 32)]);
        decode_expect("ADIW R26, 63", vec![ADIW(26, 63)]);
    }

    #[test]
    fn and() {
        decode_expect("AND R1, R1", vec![AND(1, 1)]);
        decode_expect("AND R2, R2", vec![AND(2, 2)]);
        decode_expect("AND R4, R4", vec![AND(4, 4)]);
        decode_expect("AND R8, R8", vec![AND(8, 8)]);
        decode_expect("AND R16, R16", vec![AND(16, 16)]);
        decode_expect("AND R3, R5", vec![AND(3, 5)]);
        decode_expect("AND R31, R0", vec![AND(31, 0)]);
        decode_expect("AND R0, R31", vec![AND(0, 31)]);
        decode_expect("AND R17, R17", vec![AND(17, 17)]);
    }

    #[test]
    fn andi() {
        decode_expect("ANDI R16, 16", vec![ANDI(16, 16)]);
        decode_expect("ANDI R17, 50", vec![ANDI(17, 50)]);
        decode_expect("ANDI R18, 100", vec![ANDI(18, 100)]);
        decode_expect("ANDI R20, 101", vec![ANDI(20, 101)]);
        decode_expect("ANDI R24, 240", vec![ANDI(24, 240)]);
        decode_expect("ANDI R31, 255", vec![ANDI(31, 255)]);

        decode_expect("CBR R21, 0", vec![ANDI(21, 255)]);
        decode_expect("CBR R22, 1", vec![ANDI(22, 254)]);
        decode_expect("CBR R23, 2", vec![ANDI(23, 253)]);
        decode_expect("CBR R24, 4", vec![ANDI(24, 251)]);
        decode_expect("CBR R25, 8", vec![ANDI(25, 247)]);
        decode_expect("CBR R26, 16", vec![ANDI(26, 239)]);
        decode_expect("CBR R27, 32", vec![ANDI(27, 223)]);
        decode_expect("CBR R28, 64", vec![ANDI(28, 191)]);
        decode_expect("CBR R29, 128", vec![ANDI(29, 127)]);
        decode_expect("CBR R31, 255", vec![ANDI(31, 0)]);
    }

    #[test]
    fn asr() {
        decode_expect("ASR R0", vec![ASR(0)]);
        decode_expect("ASR R1", vec![ASR(1)]);
        decode_expect("ASR R2", vec![ASR(2)]);
        decode_expect("ASR R4", vec![ASR(4)]);
        decode_expect("ASR R8", vec![ASR(8)]);
        decode_expect("ASR R16", vec![ASR(16)]);
        decode_expect("ASR R31", vec![ASR(31)]);
    }

    #[test]
    fn bclr() {
        decode_expect("BCLR 0", vec![BCLR(0)]);
        decode_expect("BCLR 1", vec![BCLR(1)]);
        decode_expect("BCLR 2", vec![BCLR(2)]);
        decode_expect("BCLR 4", vec![BCLR(4)]);
        decode_expect("BCLR 7", vec![BCLR(7)]);

        decode_expect("CLC", vec![BCLR(0)]);
        decode_expect("CLZ", vec![BCLR(1)]);
        decode_expect("CLN", vec![BCLR(2)]);
        decode_expect("CLV", vec![BCLR(3)]);
        decode_expect("CLS", vec![BCLR(4)]);
        decode_expect("CLH", vec![BCLR(5)]);
        decode_expect("CLT", vec![BCLR(6)]);
        decode_expect("CLI", vec![BCLR(7)]);
    }

    #[test]
    fn bset() {
        decode_expect("BSET 0", vec![BSET(0)]);
        decode_expect("BSET 1", vec![BSET(1)]);
        decode_expect("BSET 2", vec![BSET(2)]);
        decode_expect("BSET 4", vec![BSET(4)]);
        decode_expect("BSET 7", vec![BSET(7)]);

        decode_expect("SEC", vec![BSET(0)]);
        decode_expect("SEZ", vec![BSET(1)]);
        decode_expect("SEN", vec![BSET(2)]);
        decode_expect("SEV", vec![BSET(3)]);
        decode_expect("SES", vec![BSET(4)]);
        decode_expect("SEH", vec![BSET(5)]);
        decode_expect("SET", vec![BSET(6)]);
        decode_expect("SEI", vec![BSET(7)]);
    }

    #[test]
    fn bld() {
        decode_expect("BLD R0, 0", vec![BLD_ST(LDType::LD, 0, 0)]);
        decode_expect("BLD R1, 1", vec![BLD_ST(LDType::LD, 1, 1)]);
        decode_expect("BLD R2, 2", vec![BLD_ST(LDType::LD, 2, 2)]);
        decode_expect("BLD R4, 4", vec![BLD_ST(LDType::LD, 4, 4)]);
        decode_expect("BLD R8, 7", vec![BLD_ST(LDType::LD, 8, 7)]);
        decode_expect("BLD R16, 5", vec![BLD_ST(LDType::LD, 16, 5)]);
        decode_expect("BLD R31, 3", vec![BLD_ST(LDType::LD, 31, 3)]);
    }

    #[test]
    fn brbc() {
        decode_expect("d:BRBC 0, d", vec![BRBC_S(Clear, 0, 0)]);
        decode_expect("BRBC 1, d\nd:nop", vec![BRBC_S(Clear, 1, 1), NOP]);
        decode_expect("d:nop\nBRBC 2, d", vec![NOP, BRBC_S(Clear, 2, -1)]);
        decode_expect("BRBC 3, d\nnop\nd:nop", vec![BRBC_S(Clear, 3, 2), NOP, NOP]);
        decode_expect("d:nop\nnop\nBRBC 4, d", vec![NOP, NOP, BRBC_S(Clear, 4, -2)]);
        decode_expect("BRBC 5, d\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Clear, 5, 4), NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nBRBC 6, d", vec![ NOP, NOP, NOP, NOP, BRBC_S(Clear, 6, -4)]);
        decode_expect("BRBC 7, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Clear, 7, 8), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBC 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Clear, 3, -8)]);
        decode_expect("BRBC 5, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Clear, 5, 16), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBC 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Clear, 3, -16)]);
        decode_expect("BRBC 5, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Clear, 5, 32), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBC 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Clear, 3, -32)]);
        decode_expect("BRBC 5, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Clear, 5, 63), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBC 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Clear, 3, -63)]);
        // k=64 or k=-64 give errors: relocation truncated to fit

        decode_expect("BRCC d\nd:nop", vec![BRBC_S(Clear, 0, 1), NOP]);
        decode_expect("BRGE d\nd:nop", vec![BRBC_S(Clear, 4, 1), NOP]);
        decode_expect("BRHC d\nd:nop", vec![BRBC_S(Clear, 5, 1), NOP]);
        decode_expect("BRID d\nd:nop", vec![BRBC_S(Clear, 7, 1), NOP]);
        decode_expect("BRNE d\nd:nop", vec![BRBC_S(Clear, 1, 1), NOP]);
        decode_expect("BRPL d\nd:nop", vec![BRBC_S(Clear, 2, 1), NOP]);
        decode_expect("BRSH d\nd:nop", vec![BRBC_S(Clear, 0, 1), NOP]);
        decode_expect("BRTC d\nd:nop", vec![BRBC_S(Clear, 6, 1), NOP]);
        decode_expect("BRVC d\nd:nop", vec![BRBC_S(Clear, 3, 1), NOP]);
    }

    #[test]
    fn brbs() {
        decode_expect("d:BRBS 0, d", vec![BRBC_S(Set, 0, 0)]);
        decode_expect("BRBS 1, d\nd:nop", vec![BRBC_S(Set, 1, 1), NOP]);
        decode_expect("d:nop\nBRBS 2, d", vec![NOP, BRBC_S(Set, 2, -1)]);
        decode_expect("BRBS 3, d\nnop\nd:nop", vec![BRBC_S(Set, 3, 2), NOP, NOP]);
        decode_expect("d:nop\nnop\nBRBS 4, d", vec![NOP, NOP, BRBC_S(Set, 4, -2)]);
        decode_expect("BRBS 5, d\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Set, 5, 4), NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nBRBS 6, d", vec![ NOP, NOP, NOP, NOP, BRBC_S(Set, 6, -4)]);
        decode_expect("BRBS 7, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Set, 7, 8), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBS 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Set, 3, -8)]);
        decode_expect("BRBS 5, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Set, 5, 16), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBS 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Set, 3, -16)]);
        decode_expect("BRBS 5, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Set, 5, 32), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBS 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Set, 3, -32)]);
        decode_expect("BRBS 5, d\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nd:nop", vec![BRBC_S(Set, 5, 63), NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP]);
        decode_expect("d:nop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nnop\nBRBS 3, d", vec![NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, NOP, BRBC_S(Set, 3, -63)]);
        // k=64 or k=-64 give errors: relocation truncated to fit

        decode_expect("BRCS d\nd:nop", vec![BRBC_S(Set, 0, 1), NOP]);
        decode_expect("BREQ d\nd:nop", vec![BRBC_S(Set, 1, 1), NOP]);
        decode_expect("BRHS d\nd:nop", vec![BRBC_S(Set, 5, 1), NOP]);
        decode_expect("BRIE d\nd:nop", vec![BRBC_S(Set, 7, 1), NOP]);
        decode_expect("BRLO d\nd:nop", vec![BRBC_S(Set, 0, 1), NOP]);
        decode_expect("BRLT d\nd:nop", vec![BRBC_S(Set, 4, 1), NOP]);
        decode_expect("BRMI d\nd:nop", vec![BRBC_S(Set, 2, 1), NOP]);
        decode_expect("BRTS d\nd:nop", vec![BRBC_S(Set, 6, 1), NOP]);
        decode_expect("BRVS d\nd:nop", vec![BRBC_S(Set, 3, 1), NOP]);
    }

    #[test]
    fn bst() {
        decode_expect("BST R0, 0", vec![BLD_ST(LDType::ST, 0, 0)]);
        decode_expect("BST R1, 1", vec![BLD_ST(LDType::ST, 1, 1)]);
        decode_expect("BST R2, 2", vec![BLD_ST(LDType::ST, 2, 2)]);
        decode_expect("BST R4, 4", vec![BLD_ST(LDType::ST, 4, 4)]);
        decode_expect("BST R8, 7", vec![BLD_ST(LDType::ST, 8, 7)]);
        decode_expect("BST R16, 5", vec![BLD_ST(LDType::ST, 16, 5)]);
        decode_expect("BST R31, 3", vec![BLD_ST(LDType::ST, 31, 3)]);
    }

    #[test]
    fn call() {
        decode_expect("CALL 0", vec![CALL(0), SecondOpWord]);
        decode_expect("CALL 2", vec![CALL(1), SecondOpWord]);
        decode_expect("CALL 4", vec![CALL(2), SecondOpWord]);
        decode_expect("CALL 8", vec![CALL(4), SecondOpWord]);
        decode_expect("CALL 16", vec![CALL(8), SecondOpWord]);
        decode_expect("CALL 32", vec![CALL(16), SecondOpWord]);
        decode_expect("CALL 64", vec![CALL(32), SecondOpWord]);
        decode_expect("CALL 128", vec![CALL(64), SecondOpWord]);
        decode_expect("CALL 256", vec![CALL(128), SecondOpWord]);
        decode_expect("CALL 512", vec![CALL(256), SecondOpWord]);
        decode_expect("CALL 1024", vec![CALL(512), SecondOpWord]);
        decode_expect("CALL 2048", vec![CALL(1024), SecondOpWord]);
        decode_expect("CALL 4096", vec![CALL(2048), SecondOpWord]);
        decode_expect("CALL 8192", vec![CALL(4096), SecondOpWord]);
        decode_expect("CALL 16384", vec![CALL(8192), SecondOpWord]);
        decode_expect("CALL 32768", vec![CALL(16384), SecondOpWord]);
        decode_expect("CALL 65534", vec![CALL(32767), SecondOpWord]);
        decode_expect("CALL 12346", vec![CALL(6173), SecondOpWord]);
    }

    #[test]
    fn cbi() {
        decode_expect("CBI 0, 0", vec![C_SBI(Clear, 0, 0)]);
        decode_expect("CBI 1, 1", vec![C_SBI(Clear, 1, 1)]);
        decode_expect("CBI 2, 2", vec![C_SBI(Clear, 2, 2)]);
        decode_expect("CBI 4, 4", vec![C_SBI(Clear, 4, 4)]);
        decode_expect("CBI 8, 7", vec![C_SBI(Clear, 8, 7)]);
        decode_expect("CBI 16, 5", vec![C_SBI(Clear, 16, 5)]);
        decode_expect("CBI 31, 3", vec![C_SBI(Clear, 31, 3)]);
    }

    #[test]
    fn sbi() {
        decode_expect("SBI 0, 0", vec![C_SBI(Set, 0, 0)]);
        decode_expect("SBI 1, 1", vec![C_SBI(Set, 1, 1)]);
        decode_expect("SBI 2, 2", vec![C_SBI(Set, 2, 2)]);
        decode_expect("SBI 4, 4", vec![C_SBI(Set, 4, 4)]);
        decode_expect("SBI 8, 7", vec![C_SBI(Set, 8, 7)]);
        decode_expect("SBI 16, 5", vec![C_SBI(Set, 16, 5)]);
        decode_expect("SBI 31, 3", vec![C_SBI(Set, 31, 3)]);
    }

    #[test]
    fn com() {
        decode_expect("COM R0", vec![COM(0)]);
        decode_expect("COM R1", vec![COM(1)]);
        decode_expect("COM R2", vec![COM(2)]);
        decode_expect("COM R4", vec![COM(4)]);
        decode_expect("COM R8", vec![COM(8)]);
        decode_expect("COM R16", vec![COM(16)]);
        decode_expect("COM R31", vec![COM(31)]);
    }

    #[test]
    fn cp() {
        decode_expect("CP R1, R1", vec![CP(1, 1)]);
        decode_expect("CP R2, R2", vec![CP(2, 2)]);
        decode_expect("CP R4, R4", vec![CP(4, 4)]);
        decode_expect("CP R8, R8", vec![CP(8, 8)]);
        decode_expect("CP R16, R16", vec![CP(16, 16)]);
        decode_expect("CP R3, R5", vec![CP(3, 5)]);
        decode_expect("CP R31, R0", vec![CP(31, 0)]);
        decode_expect("CP R0, R31", vec![CP(0, 31)]);
        decode_expect("CP R17, R17", vec![CP(17, 17)]);
    }

    #[test]
    fn cpc() {
        decode_expect("CPC R1, R1", vec![CPC(1, 1)]);
        decode_expect("CPC R2, R2", vec![CPC(2, 2)]);
        decode_expect("CPC R4, R4", vec![CPC(4, 4)]);
        decode_expect("CPC R8, R8", vec![CPC(8, 8)]);
        decode_expect("CPC R16, R16", vec![CPC(16, 16)]);
        decode_expect("CPC R3, R5", vec![CPC(3, 5)]);
        decode_expect("CPC R31, R0", vec![CPC(31, 0)]);
        decode_expect("CPC R0, R31", vec![CPC(0, 31)]);
        decode_expect("CPC R17, R17", vec![CPC(17, 17)]);
    }

    #[test]
    fn cpi() {
        decode_expect("CPI R16, 0", vec![CPI(16, 0)]);
        decode_expect("CPI R20, 1", vec![CPI(20, 1)]);
        decode_expect("CPI R23, 2", vec![CPI(23, 2)]);
        decode_expect("CPI R24, 4", vec![CPI(24, 4)]);
        decode_expect("CPI R25, 8", vec![CPI(25, 8)]);
        decode_expect("CPI R26, 16", vec![CPI(26, 16)]);
        decode_expect("CPI R27, 32", vec![CPI(27, 32)]);
        decode_expect("CPI R28, 64", vec![CPI(28, 64)]);
        decode_expect("CPI R29, 128", vec![CPI(29, 128)]);
        decode_expect("CPI R31, 255", vec![CPI(31, 255)]);
    }

    #[test]
    fn cpse() {
        decode_expect("CPSE R1, R1", vec![CPSE(1, 1)]);
        decode_expect("CPSE R2, R2", vec![CPSE(2, 2)]);
        decode_expect("CPSE R4, R4", vec![CPSE(4, 4)]);
        decode_expect("CPSE R8, R8", vec![CPSE(8, 8)]);
        decode_expect("CPSE R16, R16", vec![CPSE(16, 16)]);
        decode_expect("CPSE R3, R5", vec![CPSE(3, 5)]);
        decode_expect("CPSE R31, R0", vec![CPSE(31, 0)]);
        decode_expect("CPSE R0, R31", vec![CPSE(0, 31)]);
        decode_expect("CPSE R17, R17", vec![CPSE(17, 17)]);
    }

    #[test]
    fn dec() {
        decode_expect("DEC R0", vec![DEC(0)]);
        decode_expect("DEC R1", vec![DEC(1)]);
        decode_expect("DEC R2", vec![DEC(2)]);
        decode_expect("DEC R4", vec![DEC(4)]);
        decode_expect("DEC R8", vec![DEC(8)]);
        decode_expect("DEC R16", vec![DEC(16)]);
        decode_expect("DEC R31", vec![DEC(31)]);
    }

    #[test]
    fn eor() {
        decode_expect("EOR R0, R0", vec![EOR(0, 0)]);
        decode_expect("EOR R1, R1", vec![EOR(1, 1)]);
        decode_expect("EOR R2, R2", vec![EOR(2, 2)]);
        decode_expect("EOR R4, R4", vec![EOR(4, 4)]);
        decode_expect("EOR R8, R8", vec![EOR(8, 8)]);
        decode_expect("EOR R16, R16", vec![EOR(16, 16)]);
        decode_expect("EOR R3, R5", vec![EOR(3, 5)]);
        decode_expect("EOR R31, R0", vec![EOR(31, 0)]);
        decode_expect("EOR R0, R31", vec![EOR(0, 31)]);
        decode_expect("EOR R17, R17", vec![EOR(17, 17)]);
    }

    #[test]
    fn icall() {
        decode_expect("ICALL", vec![ICALL]);
    }

    #[test]
    fn in_() {
        decode_expect("IN R0, 0", vec![IN(0, 0)]);
        decode_expect("IN R1, 1", vec![IN(1, 1)]);
        decode_expect("IN R2, 2", vec![IN(2, 2)]);
        decode_expect("IN R4, 4", vec![IN(4, 4)]);
        decode_expect("IN R8, 8", vec![IN(8, 8)]);
        decode_expect("IN R16, 16", vec![IN(16, 16)]);
        decode_expect("IN R31, 32", vec![IN(31, 32)]);
        decode_expect("IN R25, 63", vec![IN(25, 63)]);
    }

    #[test]
    fn inc() {
        decode_expect("INC R0", vec![INC(0)]);
        decode_expect("INC R2", vec![INC(2)]);
        decode_expect("INC R1", vec![INC(1)]);
        decode_expect("INC R4", vec![INC(4)]);
        decode_expect("INC R8", vec![INC(8)]);
        decode_expect("INC R16", vec![INC(16)]);
        decode_expect("INC R31", vec![INC(31)]);
    }

    #[test]
    fn jmp() {
        decode_expect("JMP 0", vec![JMP(0), SecondOpWord]);
        decode_expect("JMP 2", vec![JMP(1), SecondOpWord]);
        decode_expect("JMP 4", vec![JMP(2), SecondOpWord]);
        decode_expect("JMP 8", vec![JMP(4), SecondOpWord]);
        decode_expect("JMP 16", vec![JMP(8), SecondOpWord]);
        decode_expect("JMP 32", vec![JMP(16), SecondOpWord]);
        decode_expect("JMP 64", vec![JMP(32), SecondOpWord]);
        decode_expect("JMP 128", vec![JMP(64), SecondOpWord]);
        decode_expect("JMP 256", vec![JMP(128), SecondOpWord]);
        decode_expect("JMP 512", vec![JMP(256), SecondOpWord]);
        decode_expect("JMP 1024", vec![JMP(512), SecondOpWord]);
        decode_expect("JMP 2048", vec![JMP(1024), SecondOpWord]);
        decode_expect("JMP 4096", vec![JMP(2048), SecondOpWord]);
        decode_expect("JMP 8192", vec![JMP(4096), SecondOpWord]);
        decode_expect("JMP 16384", vec![JMP(8192), SecondOpWord]);
        decode_expect("JMP 32768", vec![JMP(16384), SecondOpWord]);
        decode_expect("JMP 65534", vec![JMP(32767), SecondOpWord]);
        decode_expect("JMP 12346", vec![JMP(6173), SecondOpWord]);
    }

    /*#[test]
    fn lac() {
        decode_expect("LAC Z, R0", vec![LAC(0)]);
        decode_expect("LAC Z, R1", vec![LAC(1)]);
        decode_expect("LAC Z, R2", vec![LAC(2)]);
        decode_expect("LAC Z, R4", vec![LAC(4)]);
        decode_expect("LAC Z, R8", vec![LAC(8)]);
        decode_expect("LAC Z, R16", vec![LAC(16)]);
        decode_expect("LAC Z, R31", vec![LAC(31)]);
    }

    #[test]
    fn las() {
        decode_expect("LAS Z, R0", vec![LAS(0)]);
        decode_expect("LAS Z, R1", vec![LAS(1)]);
        decode_expect("LAS Z, R2", vec![LAS(2)]);
        decode_expect("LAS Z, R4", vec![LAS(4)]);
        decode_expect("LAS Z, R8", vec![LAS(8)]);
        decode_expect("LAS Z, R16", vec![LAS(16)]);
        decode_expect("LAS Z, R31", vec![LAS(31)]);
    }

    #[test]
    fn lat() {
        decode_expect("LAT Z, R0", vec![LAT(0)]);
        decode_expect("LAT Z, R1", vec![LAT(1)]);
        decode_expect("LAT Z, R2", vec![LAT(2)]);
        decode_expect("LAT Z, R4", vec![LAT(4)]);
        decode_expect("LAT Z, R8", vec![LAT(8)]);
        decode_expect("LAT Z, R16", vec![LAT(16)]);
        decode_expect("LAT Z, R31", vec![LAT(31)]);
    }*/

    #[test]
    fn ld_x() {
        decode_expect("LD R0, X", vec![LD_ST(LDType::LD, 0, X, LDMode::Displacement(0))]);
        decode_expect("LD R1, X", vec![LD_ST(LDType::LD, 1, X, LDMode::Displacement(0))]);
        decode_expect("LD R2, X", vec![LD_ST(LDType::LD, 2, X, LDMode::Displacement(0))]);
        decode_expect("LD R4, X", vec![LD_ST(LDType::LD, 4, X, LDMode::Displacement(0))]);
        decode_expect("LD R8, X", vec![LD_ST(LDType::LD, 8, X, LDMode::Displacement(0))]);
        decode_expect("LD R16, X", vec![LD_ST(LDType::LD, 16, X, LDMode::Displacement(0))]);
        decode_expect("LD R31, X", vec![LD_ST(LDType::LD, 31, X, LDMode::Displacement(0))]);

        decode_expect("LD R0, X+", vec![LD_ST(LDType::LD, 0, X, LDMode::PostIncrement)]);
        decode_expect("LD R1, X+", vec![LD_ST(LDType::LD, 1, X, LDMode::PostIncrement)]);
        decode_expect("LD R2, X+", vec![LD_ST(LDType::LD, 2, X, LDMode::PostIncrement)]);
        decode_expect("LD R4, X+", vec![LD_ST(LDType::LD, 4, X, LDMode::PostIncrement)]);
        decode_expect("LD R8, X+", vec![LD_ST(LDType::LD, 8, X, LDMode::PostIncrement)]);
        decode_expect("LD R16, X+", vec![LD_ST(LDType::LD, 16, X, LDMode::PostIncrement)]);
        decode_expect("LD R31, X+", vec![LD_ST(LDType::LD, 31, X, LDMode::PostIncrement)]);

        decode_expect("LD R0, -X", vec![LD_ST(LDType::LD, 0, X, LDMode::PreDecrement)]);
        decode_expect("LD R1, -X", vec![LD_ST(LDType::LD, 1, X, LDMode::PreDecrement)]);
        decode_expect("LD R2, -X", vec![LD_ST(LDType::LD, 2, X, LDMode::PreDecrement)]);
        decode_expect("LD R4, -X", vec![LD_ST(LDType::LD, 4, X, LDMode::PreDecrement)]);
        decode_expect("LD R8, -X", vec![LD_ST(LDType::LD, 8, X, LDMode::PreDecrement)]);
        decode_expect("LD R16, -X", vec![LD_ST(LDType::LD, 16, X, LDMode::PreDecrement)]);
        decode_expect("LD R31, -X", vec![LD_ST(LDType::LD, 31, X, LDMode::PreDecrement)]);
    }

    #[test]
    fn ld_y() {
        decode_expect("LD R0, Y", vec![LD_ST(LDType::LD, 0, Y, LDMode::Displacement(0))]);
        decode_expect("LD R1, Y", vec![LD_ST(LDType::LD, 1, Y, LDMode::Displacement(0))]);
        decode_expect("LD R2, Y", vec![LD_ST(LDType::LD, 2, Y, LDMode::Displacement(0))]);
        decode_expect("LD R4, Y", vec![LD_ST(LDType::LD, 4, Y, LDMode::Displacement(0))]);
        decode_expect("LD R8, Y", vec![LD_ST(LDType::LD, 8, Y, LDMode::Displacement(0))]);
        decode_expect("LD R16, Y", vec![LD_ST(LDType::LD, 16, Y, LDMode::Displacement(0))]);
        decode_expect("LD R31, Y", vec![LD_ST(LDType::LD, 31, Y, LDMode::Displacement(0))]);

        decode_expect("LD R0, Y+", vec![LD_ST(LDType::LD, 0, Y, LDMode::PostIncrement)]);
        decode_expect("LD R1, Y+", vec![LD_ST(LDType::LD, 1, Y, LDMode::PostIncrement)]);
        decode_expect("LD R2, Y+", vec![LD_ST(LDType::LD, 2, Y, LDMode::PostIncrement)]);
        decode_expect("LD R4, Y+", vec![LD_ST(LDType::LD, 4, Y, LDMode::PostIncrement)]);
        decode_expect("LD R8, Y+", vec![LD_ST(LDType::LD, 8, Y, LDMode::PostIncrement)]);
        decode_expect("LD R16, Y+", vec![LD_ST(LDType::LD, 16, Y, LDMode::PostIncrement)]);
        decode_expect("LD R31, Y+", vec![LD_ST(LDType::LD, 31, Y, LDMode::PostIncrement)]);

        decode_expect("LD R0, -Y", vec![LD_ST(LDType::LD, 0, Y, LDMode::PreDecrement)]);
        decode_expect("LD R1, -Y", vec![LD_ST(LDType::LD, 1, Y, LDMode::PreDecrement)]);
        decode_expect("LD R2, -Y", vec![LD_ST(LDType::LD, 2, Y, LDMode::PreDecrement)]);
        decode_expect("LD R4, -Y", vec![LD_ST(LDType::LD, 4, Y, LDMode::PreDecrement)]);
        decode_expect("LD R8, -Y", vec![LD_ST(LDType::LD, 8, Y, LDMode::PreDecrement)]);
        decode_expect("LD R16, -Y", vec![LD_ST(LDType::LD, 16, Y, LDMode::PreDecrement)]);
        decode_expect("LD R31, -Y", vec![LD_ST(LDType::LD, 31, Y, LDMode::PreDecrement)]);

        decode_expect("LDD R0, Y+0", vec![LD_ST(LDType::LD, 0, Y, LDMode::Displacement(0))]);
        decode_expect("LDD R1, Y+1", vec![LD_ST(LDType::LD, 1, Y, LDMode::Displacement(1))]);
        decode_expect("LDD R2, Y+2", vec![LD_ST(LDType::LD, 2, Y, LDMode::Displacement(2))]);
        decode_expect("LDD R4, Y+4", vec![LD_ST(LDType::LD, 4, Y, LDMode::Displacement(4))]);
        decode_expect("LDD R8, Y+8", vec![LD_ST(LDType::LD, 8, Y, LDMode::Displacement(8))]);
        decode_expect("LDD R16, Y+16", vec![LD_ST(LDType::LD, 16, Y, LDMode::Displacement(16))]);
        decode_expect("LDD R31, Y+32", vec![LD_ST(LDType::LD, 31, Y, LDMode::Displacement(32))]);
        decode_expect("LDD R31, Y+63", vec![LD_ST(LDType::LD, 31, Y, LDMode::Displacement(63))]);
    }

    #[test]
    fn ld_z() {
        decode_expect("LD R0, Z", vec![LD_ST(LDType::LD, 0, Z, LDMode::Displacement(0))]);
        decode_expect("LD R1, Z", vec![LD_ST(LDType::LD, 1, Z, LDMode::Displacement(0))]);
        decode_expect("LD R2, Z", vec![LD_ST(LDType::LD, 2, Z, LDMode::Displacement(0))]);
        decode_expect("LD R4, Z", vec![LD_ST(LDType::LD, 4, Z, LDMode::Displacement(0))]);
        decode_expect("LD R8, Z", vec![LD_ST(LDType::LD, 8, Z, LDMode::Displacement(0))]);
        decode_expect("LD R16, Z", vec![LD_ST(LDType::LD, 16, Z, LDMode::Displacement(0))]);
        decode_expect("LD R31, Z", vec![LD_ST(LDType::LD, 31, Z, LDMode::Displacement(0))]);

        decode_expect("LD R0, Z+", vec![LD_ST(LDType::LD, 0, Z, LDMode::PostIncrement)]);
        decode_expect("LD R1, Z+", vec![LD_ST(LDType::LD, 1, Z, LDMode::PostIncrement)]);
        decode_expect("LD R2, Z+", vec![LD_ST(LDType::LD, 2, Z, LDMode::PostIncrement)]);
        decode_expect("LD R4, Z+", vec![LD_ST(LDType::LD, 4, Z, LDMode::PostIncrement)]);
        decode_expect("LD R8, Z+", vec![LD_ST(LDType::LD, 8, Z, LDMode::PostIncrement)]);
        decode_expect("LD R16, Z+", vec![LD_ST(LDType::LD, 16, Z, LDMode::PostIncrement)]);
        decode_expect("LD R31, Z+", vec![LD_ST(LDType::LD, 31, Z, LDMode::PostIncrement)]);

        decode_expect("LD R0, -Z", vec![LD_ST(LDType::LD, 0, Z, LDMode::PreDecrement)]);
        decode_expect("LD R1, -Z", vec![LD_ST(LDType::LD, 1, Z, LDMode::PreDecrement)]);
        decode_expect("LD R2, -Z", vec![LD_ST(LDType::LD, 2, Z, LDMode::PreDecrement)]);
        decode_expect("LD R4, -Z", vec![LD_ST(LDType::LD, 4, Z, LDMode::PreDecrement)]);
        decode_expect("LD R8, -Z", vec![LD_ST(LDType::LD, 8, Z, LDMode::PreDecrement)]);
        decode_expect("LD R16, -Z", vec![LD_ST(LDType::LD, 16, Z, LDMode::PreDecrement)]);
        decode_expect("LD R31, -Z", vec![LD_ST(LDType::LD, 31, Z, LDMode::PreDecrement)]);

        decode_expect("LDD R0, Z+0", vec![LD_ST(LDType::LD, 0, Z, LDMode::Displacement(0))]);
        decode_expect("LDD R1, Z+1", vec![LD_ST(LDType::LD, 1, Z, LDMode::Displacement(1))]);
        decode_expect("LDD R2, Z+2", vec![LD_ST(LDType::LD, 2, Z, LDMode::Displacement(2))]);
        decode_expect("LDD R4, Z+4", vec![LD_ST(LDType::LD, 4, Z, LDMode::Displacement(4))]);
        decode_expect("LDD R8, Z+8", vec![LD_ST(LDType::LD, 8, Z, LDMode::Displacement(8))]);
        decode_expect("LDD R16, Z+16", vec![LD_ST(LDType::LD, 16, Z, LDMode::Displacement(16))]);
        decode_expect("LDD R31, Z+32", vec![LD_ST(LDType::LD, 31, Z, LDMode::Displacement(32))]);
        decode_expect("LDD R31, Z+63", vec![LD_ST(LDType::LD, 31, Z, LDMode::Displacement(63))]);
    }

    #[test]
    fn st_x() {
        decode_expect("ST X, R0", vec![LD_ST(LDType::ST, 0, X, LDMode::Displacement(0))]);
        decode_expect("ST X, R1", vec![LD_ST(LDType::ST, 1, X, LDMode::Displacement(0))]);
        decode_expect("ST X, R2", vec![LD_ST(LDType::ST, 2, X, LDMode::Displacement(0))]);
        decode_expect("ST X, R4", vec![LD_ST(LDType::ST, 4, X, LDMode::Displacement(0))]);
        decode_expect("ST X, R8", vec![LD_ST(LDType::ST, 8, X, LDMode::Displacement(0))]);
        decode_expect("ST X, R16", vec![LD_ST(LDType::ST, 16, X, LDMode::Displacement(0))]);
        decode_expect("ST X, R31", vec![LD_ST(LDType::ST, 31, X, LDMode::Displacement(0))]);

        decode_expect("ST X+, R0", vec![LD_ST(LDType::ST, 0, X, LDMode::PostIncrement)]);
        decode_expect("ST X+, R1", vec![LD_ST(LDType::ST, 1, X, LDMode::PostIncrement)]);
        decode_expect("ST X+, R2", vec![LD_ST(LDType::ST, 2, X, LDMode::PostIncrement)]);
        decode_expect("ST X+, R4", vec![LD_ST(LDType::ST, 4, X, LDMode::PostIncrement)]);
        decode_expect("ST X+, R8", vec![LD_ST(LDType::ST, 8, X, LDMode::PostIncrement)]);
        decode_expect("ST X+, R16", vec![LD_ST(LDType::ST, 16, X, LDMode::PostIncrement)]);
        decode_expect("ST X+, R31", vec![LD_ST(LDType::ST, 31, X, LDMode::PostIncrement)]);

        decode_expect("ST -X, R0", vec![LD_ST(LDType::ST, 0, X, LDMode::PreDecrement)]);
        decode_expect("ST -X, R1", vec![LD_ST(LDType::ST, 1, X, LDMode::PreDecrement)]);
        decode_expect("ST -X, R2", vec![LD_ST(LDType::ST, 2, X, LDMode::PreDecrement)]);
        decode_expect("ST -X, R4", vec![LD_ST(LDType::ST, 4, X, LDMode::PreDecrement)]);
        decode_expect("ST -X, R8", vec![LD_ST(LDType::ST, 8, X, LDMode::PreDecrement)]);
        decode_expect("ST -X, R16", vec![LD_ST(LDType::ST, 16, X, LDMode::PreDecrement)]);
        decode_expect("ST -X, R31", vec![LD_ST(LDType::ST, 31, X, LDMode::PreDecrement)]);
    }

    #[test]
    fn st_y() {
        decode_expect("ST Y, R0", vec![LD_ST(LDType::ST, 0, Y, LDMode::Displacement(0))]);
        decode_expect("ST Y, R1", vec![LD_ST(LDType::ST, 1, Y, LDMode::Displacement(0))]);
        decode_expect("ST Y, R2", vec![LD_ST(LDType::ST, 2, Y, LDMode::Displacement(0))]);
        decode_expect("ST Y, R4", vec![LD_ST(LDType::ST, 4, Y, LDMode::Displacement(0))]);
        decode_expect("ST Y, R8", vec![LD_ST(LDType::ST, 8, Y, LDMode::Displacement(0))]);
        decode_expect("ST Y, R16", vec![LD_ST(LDType::ST, 16, Y, LDMode::Displacement(0))]);
        decode_expect("ST Y, R31", vec![LD_ST(LDType::ST, 31, Y, LDMode::Displacement(0))]);

        decode_expect("ST Y+, R0", vec![LD_ST(LDType::ST, 0, Y, LDMode::PostIncrement)]);
        decode_expect("ST Y+, R1", vec![LD_ST(LDType::ST, 1, Y, LDMode::PostIncrement)]);
        decode_expect("ST Y+, R2", vec![LD_ST(LDType::ST, 2, Y, LDMode::PostIncrement)]);
        decode_expect("ST Y+, R4", vec![LD_ST(LDType::ST, 4, Y, LDMode::PostIncrement)]);
        decode_expect("ST Y+, R8", vec![LD_ST(LDType::ST, 8, Y, LDMode::PostIncrement)]);
        decode_expect("ST Y+, R16", vec![LD_ST(LDType::ST, 16, Y, LDMode::PostIncrement)]);
        decode_expect("ST Y+, R31", vec![LD_ST(LDType::ST, 31, Y, LDMode::PostIncrement)]);

        decode_expect("ST -Y, R0", vec![LD_ST(LDType::ST, 0, Y, LDMode::PreDecrement)]);
        decode_expect("ST -Y, R1", vec![LD_ST(LDType::ST, 1, Y, LDMode::PreDecrement)]);
        decode_expect("ST -Y, R2", vec![LD_ST(LDType::ST, 2, Y, LDMode::PreDecrement)]);
        decode_expect("ST -Y, R4", vec![LD_ST(LDType::ST, 4, Y, LDMode::PreDecrement)]);
        decode_expect("ST -Y, R8", vec![LD_ST(LDType::ST, 8, Y, LDMode::PreDecrement)]);
        decode_expect("ST -Y, R16", vec![LD_ST(LDType::ST, 16, Y, LDMode::PreDecrement)]);
        decode_expect("ST -Y, R31", vec![LD_ST(LDType::ST, 31, Y, LDMode::PreDecrement)]);

        decode_expect("STD Y+0, R0", vec![LD_ST(LDType::ST, 0, Y, LDMode::Displacement(0))]);
        decode_expect("STD Y+1, R1", vec![LD_ST(LDType::ST, 1, Y, LDMode::Displacement(1))]);
        decode_expect("STD Y+2, R2", vec![LD_ST(LDType::ST, 2, Y, LDMode::Displacement(2))]);
        decode_expect("STD Y+4, R4", vec![LD_ST(LDType::ST, 4, Y, LDMode::Displacement(4))]);
        decode_expect("STD Y+8, R8", vec![LD_ST(LDType::ST, 8, Y, LDMode::Displacement(8))]);
        decode_expect("STD Y+16, R16", vec![LD_ST(LDType::ST, 16, Y, LDMode::Displacement(16))]);
        decode_expect("STD Y+32, R31", vec![LD_ST(LDType::ST, 31, Y, LDMode::Displacement(32))]);
        decode_expect("STD Y+63, R31", vec![LD_ST(LDType::ST, 31, Y, LDMode::Displacement(63))]);
    }

    #[test]
    fn st_z() {
        decode_expect("ST Z, R0", vec![LD_ST(LDType::ST, 0, Z, LDMode::Displacement(0))]);
        decode_expect("ST Z, R1", vec![LD_ST(LDType::ST, 1, Z, LDMode::Displacement(0))]);
        decode_expect("ST Z, R2", vec![LD_ST(LDType::ST, 2, Z, LDMode::Displacement(0))]);
        decode_expect("ST Z, R4", vec![LD_ST(LDType::ST, 4, Z, LDMode::Displacement(0))]);
        decode_expect("ST Z, R8", vec![LD_ST(LDType::ST, 8, Z, LDMode::Displacement(0))]);
        decode_expect("ST Z, R16", vec![LD_ST(LDType::ST, 16, Z, LDMode::Displacement(0))]);
        decode_expect("ST Z, R31", vec![LD_ST(LDType::ST, 31, Z, LDMode::Displacement(0))]);

        decode_expect("ST Z+, R0", vec![LD_ST(LDType::ST, 0, Z, LDMode::PostIncrement)]);
        decode_expect("ST Z+, R1", vec![LD_ST(LDType::ST, 1, Z, LDMode::PostIncrement)]);
        decode_expect("ST Z+, R2", vec![LD_ST(LDType::ST, 2, Z, LDMode::PostIncrement)]);
        decode_expect("ST Z+, R4", vec![LD_ST(LDType::ST, 4, Z, LDMode::PostIncrement)]);
        decode_expect("ST Z+, R8", vec![LD_ST(LDType::ST, 8, Z, LDMode::PostIncrement)]);
        decode_expect("ST Z+, R16", vec![LD_ST(LDType::ST, 16, Z, LDMode::PostIncrement)]);
        decode_expect("ST Z+, R31", vec![LD_ST(LDType::ST, 31, Z, LDMode::PostIncrement)]);

        decode_expect("ST -Z, R0", vec![LD_ST(LDType::ST, 0, Z, LDMode::PreDecrement)]);
        decode_expect("ST -Z, R1", vec![LD_ST(LDType::ST, 1, Z, LDMode::PreDecrement)]);
        decode_expect("ST -Z, R2", vec![LD_ST(LDType::ST, 2, Z, LDMode::PreDecrement)]);
        decode_expect("ST -Z, R4", vec![LD_ST(LDType::ST, 4, Z, LDMode::PreDecrement)]);
        decode_expect("ST -Z, R8", vec![LD_ST(LDType::ST, 8, Z, LDMode::PreDecrement)]);
        decode_expect("ST -Z, R16", vec![LD_ST(LDType::ST, 16, Z, LDMode::PreDecrement)]);
        decode_expect("ST -Z, R31", vec![LD_ST(LDType::ST, 31, Z, LDMode::PreDecrement)]);

        decode_expect("STD Z+0, R0", vec![LD_ST(LDType::ST, 0, Z, LDMode::Displacement(0))]);
        decode_expect("STD Z+1, R1", vec![LD_ST(LDType::ST, 1, Z, LDMode::Displacement(1))]);
        decode_expect("STD Z+2, R2", vec![LD_ST(LDType::ST, 2, Z, LDMode::Displacement(2))]);
        decode_expect("STD Z+4, R4", vec![LD_ST(LDType::ST, 4, Z, LDMode::Displacement(4))]);
        decode_expect("STD Z+8, R8", vec![LD_ST(LDType::ST, 8, Z, LDMode::Displacement(8))]);
        decode_expect("STD Z+16, R16", vec![LD_ST(LDType::ST, 16, Z, LDMode::Displacement(16))]);
        decode_expect("STD Z+32, R31", vec![LD_ST(LDType::ST, 31, Z, LDMode::Displacement(32))]);
        decode_expect("STD Z+63, R31", vec![LD_ST(LDType::ST, 31, Z, LDMode::Displacement(63))]);
    }

    #[test]
    fn ldi() {
        decode_expect("LDI R16, 0", vec![LDI(16, 0)]);
        decode_expect("LDI R20, 1", vec![LDI(20, 1)]);
        decode_expect("LDI R23, 2", vec![LDI(23, 2)]);
        decode_expect("LDI R24, 4", vec![LDI(24, 4)]);
        decode_expect("LDI R25, 8", vec![LDI(25, 8)]);
        decode_expect("LDI R26, 16", vec![LDI(26, 16)]);
        decode_expect("LDI R27, 32", vec![LDI(27, 32)]);
        decode_expect("LDI R28, 64", vec![LDI(28, 64)]);
        decode_expect("LDI R29, 128", vec![LDI(29, 128)]);
        decode_expect("LDI R31, 255", vec![LDI(31, 255)]);
    }

    #[test]
    fn lds() {
        decode_expect("LDS R0, 0", vec![LD_STS(LDType::LD, 0, 0), SecondOpWord]);
        decode_expect("LDS R1, 1", vec![LD_STS(LDType::LD, 1, 1), SecondOpWord]);
        decode_expect("LDS R2, 2", vec![LD_STS(LDType::LD, 2, 2), SecondOpWord]);
        decode_expect("LDS R4, 4", vec![LD_STS(LDType::LD, 4, 4), SecondOpWord]);
        decode_expect("LDS R8, 8", vec![LD_STS(LDType::LD, 8, 8), SecondOpWord]);
        decode_expect("LDS R16, 16", vec![LD_STS(LDType::LD, 16, 16), SecondOpWord]);
        decode_expect("LDS R31, 32", vec![LD_STS(LDType::LD, 31, 32), SecondOpWord]);
        decode_expect("LDS R31, 64", vec![LD_STS(LDType::LD, 31, 64), SecondOpWord]);
        decode_expect("LDS R31, 128", vec![LD_STS(LDType::LD, 31, 128), SecondOpWord]);
        decode_expect("LDS R31, 256", vec![LD_STS(LDType::LD, 31, 256), SecondOpWord]);
        decode_expect("LDS R31, 512", vec![LD_STS(LDType::LD, 31, 512), SecondOpWord]);
        decode_expect("LDS R31, 1024", vec![LD_STS(LDType::LD, 31, 1024), SecondOpWord]);
        decode_expect("LDS R31, 2048", vec![LD_STS(LDType::LD, 31, 2048), SecondOpWord]);
        decode_expect("LDS R31, 65535", vec![LD_STS(LDType::LD, 31, 65535), SecondOpWord]);
    }

    #[test]
    fn sts() {
        decode_expect("STS 0, R0", vec![LD_STS(LDType::ST, 0, 0), SecondOpWord]);
        decode_expect("STS 1, R1", vec![LD_STS(LDType::ST, 1, 1), SecondOpWord]);
        decode_expect("STS 2, R2", vec![LD_STS(LDType::ST, 2, 2), SecondOpWord]);
        decode_expect("STS 4, R4", vec![LD_STS(LDType::ST, 4, 4), SecondOpWord]);
        decode_expect("STS 8, R8", vec![LD_STS(LDType::ST, 8, 8), SecondOpWord]);
        decode_expect("STS 16, R16", vec![LD_STS(LDType::ST, 16, 16), SecondOpWord]);
        decode_expect("STS 32, R31", vec![LD_STS(LDType::ST, 31, 32), SecondOpWord]);
        decode_expect("STS 64, R31", vec![LD_STS(LDType::ST, 31, 64), SecondOpWord]);
        decode_expect("STS 128, R31", vec![LD_STS(LDType::ST, 31, 128), SecondOpWord]);
        decode_expect("STS 256, R31", vec![LD_STS(LDType::ST, 31, 256), SecondOpWord]);
        decode_expect("STS 512, R31", vec![LD_STS(LDType::ST, 31, 512), SecondOpWord]);
        decode_expect("STS 1024, R31", vec![LD_STS(LDType::ST, 31, 1024), SecondOpWord]);
        decode_expect("STS 2048, R31", vec![LD_STS(LDType::ST, 31, 2048), SecondOpWord]);
        decode_expect("STS 65535, R31", vec![LD_STS(LDType::ST, 31, 65535), SecondOpWord]);
    }

    #[test]
    fn lpm() {
        decode_expect("LPM", vec![LPM(0, LPMType::Z)]);

        decode_expect("LPM R0, Z", vec![LPM(0, LPMType::Z)]);
        decode_expect("LPM R1, Z", vec![LPM(1, LPMType::Z)]);
        decode_expect("LPM R2, Z", vec![LPM(2, LPMType::Z)]);
        decode_expect("LPM R4, Z", vec![LPM(4, LPMType::Z)]);
        decode_expect("LPM R8, Z", vec![LPM(8, LPMType::Z)]);
        decode_expect("LPM R16, Z", vec![LPM(16, LPMType::Z)]);
        decode_expect("LPM R31, Z", vec![LPM(31, LPMType::Z)]);

        decode_expect("LPM R0, Z+", vec![LPM(0, LPMType::ZPostIncrement)]);
        decode_expect("LPM R1, Z+", vec![LPM(1, LPMType::ZPostIncrement)]);
        decode_expect("LPM R2, Z+", vec![LPM(2, LPMType::ZPostIncrement)]);
        decode_expect("LPM R4, Z+", vec![LPM(4, LPMType::ZPostIncrement)]);
        decode_expect("LPM R8, Z+", vec![LPM(8, LPMType::ZPostIncrement)]);
        decode_expect("LPM R16, Z+", vec![LPM(16, LPMType::ZPostIncrement)]);
        decode_expect("LPM R31, Z+", vec![LPM(31, LPMType::ZPostIncrement)]);
    }

    #[test]
    fn lsr() {
        decode_expect("LSR R0", vec![LSR(0)]);
        decode_expect("LSR R1", vec![LSR(1)]);
        decode_expect("LSR R2", vec![LSR(2)]);
        decode_expect("LSR R4", vec![LSR(4)]);
        decode_expect("LSR R8", vec![LSR(8)]);
        decode_expect("LSR R16", vec![LSR(16)]);
        decode_expect("LSR R31", vec![LSR(31)]);
    }

    #[test]
    fn mov() {
        decode_expect("MOV R1, R1", vec![MOV(1, 1)]);
        decode_expect("MOV R2, R2", vec![MOV(2, 2)]);
        decode_expect("MOV R4, R4", vec![MOV(4, 4)]);
        decode_expect("MOV R8, R8", vec![MOV(8, 8)]);
        decode_expect("MOV R16, R16", vec![MOV(16, 16)]);
        decode_expect("MOV R3, R5", vec![MOV(3, 5)]);
        decode_expect("MOV R31, R0", vec![MOV(31, 0)]);
        decode_expect("MOV R0, R31", vec![MOV(0, 31)]);
        decode_expect("MOV R17, R17", vec![MOV(17, 17)]);
    }

    #[test]
    fn movw() {
        decode_expect("MOVW R0, R0", vec![MOVW(0, 0)]);
        decode_expect("MOVW R2, R2", vec![MOVW(2, 2)]);
        decode_expect("MOVW R4, R4", vec![MOVW(4, 4)]);
        decode_expect("MOVW R8, R8", vec![MOVW(8, 8)]);
        decode_expect("MOVW R16, R16", vec![MOVW(16, 16)]);
        decode_expect("MOVW R30, R30", vec![MOVW(30, 30)]);
    }

    #[test]
    fn mul() {
        decode_expect("MUL R1, R1", vec![MUL(1, 1)]);
        decode_expect("MUL R2, R2", vec![MUL(2, 2)]);
        decode_expect("MUL R4, R4", vec![MUL(4, 4)]);
        decode_expect("MUL R8, R8", vec![MUL(8, 8)]);
        decode_expect("MUL R16, R16", vec![MUL(16, 16)]);
        decode_expect("MUL R3, R5", vec![MUL(3, 5)]);
        decode_expect("MUL R31, R0", vec![MUL(31, 0)]);
        decode_expect("MUL R0, R31", vec![MUL(0, 31)]);
        decode_expect("MUL R17, R17", vec![MUL(17, 17)]);
    }

    #[test]
    fn neg() {
        decode_expect("NEG R0", vec![NEG(0)]);
        decode_expect("NEG R1", vec![NEG(1)]);
        decode_expect("NEG R2", vec![NEG(2)]);
        decode_expect("NEG R4", vec![NEG(4)]);
        decode_expect("NEG R8", vec![NEG(8)]);
        decode_expect("NEG R16", vec![NEG(16)]);
        decode_expect("NEG R31", vec![NEG(31)]);
    }

    #[test]
    fn or() {
        decode_expect("OR R0, R0", vec![OR(0, 0)]);
        decode_expect("OR R1, R1", vec![OR(1, 1)]);
        decode_expect("OR R2, R2", vec![OR(2, 2)]);
        decode_expect("OR R4, R4", vec![OR(4, 4)]);
        decode_expect("OR R8, R8", vec![OR(8, 8)]);
        decode_expect("OR R16, R16", vec![OR(16, 16)]);
        decode_expect("OR R3, R5", vec![OR(3, 5)]);
        decode_expect("OR R31, R0", vec![OR(31, 0)]);
        decode_expect("OR R0, R31", vec![OR(0, 31)]);
        decode_expect("OR R17, R17", vec![OR(17, 17)]);
    }

    #[test]
    fn ori() {
        decode_expect("ORI R16, 16", vec![ORI(16, 16)]);
        decode_expect("ORI R17, 50", vec![ORI(17, 50)]);
        decode_expect("ORI R18, 100", vec![ORI(18, 100)]);
        decode_expect("ORI R20, 101", vec![ORI(20, 101)]);
        decode_expect("ORI R24, 240", vec![ORI(24, 240)]);
        decode_expect("ORI R31, 255", vec![ORI(31, 255)]);
    }

    #[test]
    fn out_() {
        decode_expect("OUT 0, R0", vec![OUT(0, 0)]);
        decode_expect("OUT 1, R1", vec![OUT(1, 1)]);
        decode_expect("OUT 2, R2", vec![OUT(2, 2)]);
        decode_expect("OUT 4, R4", vec![OUT(4, 4)]);
        decode_expect("OUT 8, R8", vec![OUT(8, 8)]);
        decode_expect("OUT 16, R16", vec![OUT(16, 16)]);
        decode_expect("OUT 32, R31", vec![OUT(31, 32)]);
        decode_expect("OUT 63, R25", vec![OUT(25, 63)]);
    }

    #[test]
    fn pop() {
        decode_expect("POP R0", vec![POP(0)]);
        decode_expect("POP R1", vec![POP(1)]);
        decode_expect("POP R2", vec![POP(2)]);
        decode_expect("POP R4", vec![POP(4)]);
        decode_expect("POP R8", vec![POP(8)]);
        decode_expect("POP R16", vec![POP(16)]);
        decode_expect("POP R31", vec![POP(31)]);
    }

    #[test]
    fn push() {
        decode_expect("PUSH R0", vec![PUSH(0)]);
        decode_expect("PUSH R1", vec![PUSH(1)]);
        decode_expect("PUSH R2", vec![PUSH(2)]);
        decode_expect("PUSH R4", vec![PUSH(4)]);
        decode_expect("PUSH R8", vec![PUSH(8)]);
        decode_expect("PUSH R16", vec![PUSH(16)]);
        decode_expect("PUSH R31", vec![PUSH(31)]);
    }

    #[test]
    fn rcall() {
        decode_expect("d:RCALL d", vec![RCALL(0)]);
        decode_expect("d:nop\nRCALL d", vec![NOP, RCALL(-1)]);
        decode_expect("RCALL d\nd:nop", vec![RCALL(1), NOP]);
        decode_expect("d:nop\nnop\nRCALL d", vec![NOP, NOP, RCALL(-2)]);
        decode_expect("RCALL d\nnop\nd:nop", vec![RCALL(2), NOP, NOP]);
    }

    #[test]
    fn ret() {
        decode_expect("RET", vec![RET]);
    }

    #[test]
    fn reti() {
        decode_expect("RETI", vec![RETI]);
    }

    #[test]
    fn rjmp() {
        decode_expect("d:rjmp d", vec![RJMP(0)]);
        decode_expect("d:nop\nrjmp d", vec![NOP, RJMP(-1)]);
        decode_expect("rjmp d\nd:nop", vec![RJMP(1), NOP]);
        decode_expect("d:nop\nnop\nrjmp d", vec![NOP, NOP, RJMP(-2)]);
        decode_expect("rjmp d\nnop\nd:nop", vec![RJMP(2), NOP, NOP]);
    }

    #[test]
    fn ror() {
        decode_expect("ROR R0", vec![ROR(0)]);
        decode_expect("ROR R1", vec![ROR(1)]);
        decode_expect("ROR R2", vec![ROR(2)]);
        decode_expect("ROR R4", vec![ROR(4)]);
        decode_expect("ROR R8", vec![ROR(8)]);
        decode_expect("ROR R16", vec![ROR(16)]);
        decode_expect("ROR R31", vec![ROR(31)]);
    }

    #[test]
    fn sbc() {
        decode_expect("SBC R1, R1", vec![SBC(1, 1)]);
        decode_expect("SBC R2, R2", vec![SBC(2, 2)]);
        decode_expect("SBC R4, R4", vec![SBC(4, 4)]);
        decode_expect("SBC R8, R8", vec![SBC(8, 8)]);
        decode_expect("SBC R16, R16", vec![SBC(16, 16)]);
        decode_expect("SBC R3, R5", vec![SBC(3, 5)]);
        decode_expect("SBC R31, R0", vec![SBC(31, 0)]);
        decode_expect("SBC R0, R31", vec![SBC(0, 31)]);
        decode_expect("SBC R17, R17", vec![SBC(17, 17)]);
    }

    #[test]
    fn sub() {
        decode_expect("SUB R1, R1", vec![SUB(1, 1)]);
        decode_expect("SUB R2, R2", vec![SUB(2, 2)]);
        decode_expect("SUB R4, R4", vec![SUB(4, 4)]);
        decode_expect("SUB R8, R8", vec![SUB(8, 8)]);
        decode_expect("SUB R16, R16", vec![SUB(16, 16)]);
        decode_expect("SUB R3, R5", vec![SUB(3, 5)]);
        decode_expect("SUB R31, R0", vec![SUB(31, 0)]);
        decode_expect("SUB R0, R31", vec![SUB(0, 31)]);
        decode_expect("SUB R17, R17", vec![SUB(17, 17)]);
    }

    #[test]
    fn sbci() {
        decode_expect("SBCI R16, 0", vec![SBCI(16, 0)]);
        decode_expect("SBCI R20, 1", vec![SBCI(20, 1)]);
        decode_expect("SBCI R23, 2", vec![SBCI(23, 2)]);
        decode_expect("SBCI R24, 4", vec![SBCI(24, 4)]);
        decode_expect("SBCI R25, 8", vec![SBCI(25, 8)]);
        decode_expect("SBCI R26, 16", vec![SBCI(26, 16)]);
        decode_expect("SBCI R27, 32", vec![SBCI(27, 32)]);
        decode_expect("SBCI R28, 64", vec![SBCI(28, 64)]);
        decode_expect("SBCI R29, 128", vec![SBCI(29, 128)]);
        decode_expect("SBCI R31, 255", vec![SBCI(31, 255)]);
    }

    #[test]
    fn sbic() {
        decode_expect("SBIC 0, 0", vec![SBIC_S(Clear, 0, 0)]);
        decode_expect("SBIC 1, 1", vec![SBIC_S(Clear, 1, 1)]);
        decode_expect("SBIC 2, 2", vec![SBIC_S(Clear, 2, 2)]);
        decode_expect("SBIC 4, 4", vec![SBIC_S(Clear, 4, 4)]);
        decode_expect("SBIC 8, 7", vec![SBIC_S(Clear, 8, 7)]);
        decode_expect("SBIC 16, 5", vec![SBIC_S(Clear, 16, 5)]);
        decode_expect("SBIC 31, 3", vec![SBIC_S(Clear, 31, 3)]);
    }

    #[test]
    fn sbis() {
        decode_expect("SBIS 0, 0", vec![SBIC_S(Set, 0, 0)]);
        decode_expect("SBIS 1, 1", vec![SBIC_S(Set, 1, 1)]);
        decode_expect("SBIS 2, 2", vec![SBIC_S(Set, 2, 2)]);
        decode_expect("SBIS 4, 4", vec![SBIC_S(Set, 4, 4)]);
        decode_expect("SBIS 8, 7", vec![SBIC_S(Set, 8, 7)]);
        decode_expect("SBIS 16, 5", vec![SBIC_S(Set, 16, 5)]);
        decode_expect("SBIS 31, 3", vec![SBIC_S(Set, 31, 3)]);
    }

    #[test]
    fn sbiw() {
        decode_expect("SBIW R24, 0", vec![SBIW(24, 0)]);
        decode_expect("SBIW R26, 1", vec![SBIW(26, 1)]);
        decode_expect("SBIW R28, 2", vec![SBIW(28, 2)]);
        decode_expect("SBIW R30, 4", vec![SBIW(30, 4)]);
        decode_expect("SBIW R26, 8", vec![SBIW(26, 8)]);
        decode_expect("SBIW R26, 16", vec![SBIW(26, 16)]);
        decode_expect("SBIW R26, 32", vec![SBIW(26, 32)]);
        decode_expect("SBIW R26, 63", vec![SBIW(26, 63)]);
    }

    #[test]
    fn sbrc() {
        decode_expect("SBRC R0, 0", vec![SBR(Clear, 0, 0)]);
        decode_expect("SBRC R1, 1", vec![SBR(Clear, 1, 1)]);
        decode_expect("SBRC R2, 2", vec![SBR(Clear, 2, 2)]);
        decode_expect("SBRC R4, 4", vec![SBR(Clear, 4, 4)]);
        decode_expect("SBRC R8, 7", vec![SBR(Clear, 8, 7)]);
        decode_expect("SBRC R16, 5", vec![SBR(Clear, 16, 5)]);
        decode_expect("SBRC R31, 3", vec![SBR(Clear, 31, 3)]);
    }

    #[test]
    fn sbrs() {
        decode_expect("SBRS R0, 0", vec![SBR(Set, 0, 0)]);
        decode_expect("SBRS R1, 1", vec![SBR(Set, 1, 1)]);
        decode_expect("SBRS R2, 2", vec![SBR(Set, 2, 2)]);
        decode_expect("SBRS R4, 4", vec![SBR(Set, 4, 4)]);
        decode_expect("SBRS R8, 7", vec![SBR(Set, 8, 7)]);
        decode_expect("SBRS R16, 5", vec![SBR(Set, 16, 5)]);
        decode_expect("SBRS R31, 3", vec![SBR(Set, 31, 3)]);
    }

    #[test]
    fn sleep() {
        decode_expect("SLEEP", vec![SLEEP]);
    }

    #[test]
    fn subi() {
        decode_expect("SUBI R16, 0", vec![SUBI(16, 0)]);
        decode_expect("SUBI R20, 1", vec![SUBI(20, 1)]);
        decode_expect("SUBI R23, 2", vec![SUBI(23, 2)]);
        decode_expect("SUBI R24, 4", vec![SUBI(24, 4)]);
        decode_expect("SUBI R25, 8", vec![SUBI(25, 8)]);
        decode_expect("SUBI R26, 16", vec![SUBI(26, 16)]);
        decode_expect("SUBI R27, 32", vec![SUBI(27, 32)]);
        decode_expect("SUBI R28, 64", vec![SUBI(28, 64)]);
        decode_expect("SUBI R29, 128", vec![SUBI(29, 128)]);
        decode_expect("SUBI R31, 255", vec![SUBI(31, 255)]);
    }

    #[test]
    fn swap() {
        decode_expect("SWAP R0", vec![SWAP(0)]);
        decode_expect("SWAP R1", vec![SWAP(1)]);
        decode_expect("SWAP R2", vec![SWAP(2)]);
        decode_expect("SWAP R4", vec![SWAP(4)]);
        decode_expect("SWAP R8", vec![SWAP(8)]);
        decode_expect("SWAP R16", vec![SWAP(16)]);
        decode_expect("SWAP R31", vec![SWAP(31)]);
    }

    fn decode_expect(code: &str, expect: Vec<Instruction>) {
        let dec = decode(assemble(code).iter().map(|i| *i))
            .collect::<Vec<Instruction>>();

        // println!("Expected: {:?}", expect);
        // println!("Is: {:?}", dec);

        assert_eq!(expect, dec);
    }

}
