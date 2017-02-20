// use std::error;
// use std::result;
// use std::fmt;

pub const X: Register = 26;
pub const Y: Register = 28;
pub const Z: Register = 30;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    UnknownOp(u16),
    IncompleteOp(u32),
    SecondOpWord,

    ADC(Register, Register),
    ADD(Register, Register),
    ADIW(Register, u8),
    AND(Register, Register),
    ANDI(Register, u8),
    ASR(Register),
    BCLR(SREG),
    BLD_ST(LDType, Register, u8),
    BRBC_S(SetClear, SREG, i8),
    BSET(SREG),
    CALL(u32),
    C_SBI(SetClear, u8, u8),
    COM(Register),
    CP(Register, Register),
    CPC(Register, Register),
    CPI(Register, u8),
    CPSE(Register, Register),
    DEC(Register),
    EOR(Register, Register),
    ICALL,
    IN(Register, u8),
    INC(Register),
    JMP(u32),
    // LAC(Register), // not supported on atmega32
    // LAS(Register), // not supported on atmega32
    // LAT(Register), // not supported on atmega32
    LD_ST(LDType, Register, Register, LDMode),
    LD_STS(LDType, Register, u16),
    LDI(Register, u8),
    LPM(Register,LPMType),
    LSR(Register),
    MOV(Register, Register),
    MOVW(Register, Register),
    MUL(Register, Register),
    NEG(Register),
    NOP,
    OR(Register, Register),
    ORI(Register, u8),
    OUT(Register, u8),
    POP(Register),
    PUSH(Register),
    RCALL(i16),
    RET,
    RETI,
    RJMP(i16),
    ROR(Register),
    SBC(Register, Register),
    SBCI(Register, u8),
    SBIC_S(SetClear, u8, u8),
    SBIW(Register, u8),
    SBR(SetClear, Register, u8),
    SLEEP,
    SUB(Register, Register),
    SUBI(Register, u8),
    SWAP(Register)
}

/// a register by index
/// Range: 0-31
pub type Register = u8;
/// index into the sreg-register
/// Range: 0-7
pub type SREG = u8;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum LDMode {
    PostIncrement,
    PreDecrement,
    Displacement(u8)
}

impl LDMode {
    #[inline(always)]
    #[allow(dead_code)]
    pub fn as_u16(&self) -> u16 {
        match *self {
            LDMode::PostIncrement => 0,
            LDMode::PreDecrement => 1,
            LDMode::Displacement(d) => d as u16 + 2
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn from_u16(val: u16) -> LDMode {
        if val == 0 {
            LDMode::PostIncrement
        } else if val == 1 {
            LDMode::PreDecrement
        } else {
            LDMode::Displacement((val - 2) as u8)
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum LDType {
    LD, ST
}

impl LDType {
    #[inline(always)]
    #[allow(dead_code)]
    pub fn as_u8(&self) -> u8 {
        match *self {
            LDType::LD => 0,
            LDType::ST => 1,
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn from_u8(val: u8) -> LDType {
        match val {
            0 => LDType::LD,
            1 => LDType::ST,
            _ => panic!("Unknown val for from u8"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum LPMType {
    Z,
    ZPostIncrement
}

impl LPMType {
    #[inline(always)]
    #[allow(dead_code)]
    pub fn as_u8(&self) -> u8 {
        match *self {
            LPMType::Z => 0,
            LPMType::ZPostIncrement => 1,
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn from_u8(val: u8) -> LPMType {
        match val {
            0 => LPMType::Z,
            1 => LPMType::ZPostIncrement,
            _ => panic!("Unknown val for from u8"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SetClear {
    Set, Clear
}

impl SetClear {
    #[inline(always)]
    pub fn as_u8(&self) -> u8 {
        match *self {
            SetClear::Set => 1,
            SetClear::Clear => 0,
        }
    }
}
