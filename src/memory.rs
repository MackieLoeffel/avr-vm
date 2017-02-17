use data::Instruction;
use data::Instruction::NOP;
use decoder::decode;
use std::fs::File;
use std::ffi::OsString;
use std::io::prelude::*;
use std::char;
use io::IO;
use ports::{Port, adc_write};

const SRAM_SIZE: usize = 2144;
const PROGRAM_SIZE: usize = 32 * 1 << 10;
const MAX_INSTRUCTIONS: usize = PROGRAM_SIZE >> 1;
const REGISTER_OFFSET: u8 = 0;
const NUM_REGISTER: u8 = 0x20;
const IO_REGISTER_OFFSET: u8 = NUM_REGISTER;
const NUM_IO_REGISTER: u8 = 0x40;
const FLAGS_REG: u8 = 0x3f;
const SP_REG: u8 = 0x3d;
const UDR: u16 = 0x2C;

pub struct Memory<'a> {
    code: [Instruction; MAX_INSTRUCTIONS],
    program: [u8; PROGRAM_SIZE],
    data: [u8; SRAM_SIZE],
    io: Option<&'a IO>,
    ports: [Port<'a>; 4],
}

impl<'a> Memory<'a> {
    pub fn new(file: OsString, io: Option<&'a IO>) -> Memory<'a> {
        let mut output = File::open(file).unwrap();
        let mut bytes = Vec::new();
        output.read_to_end(&mut bytes).unwrap();
        assert!(bytes.len() < PROGRAM_SIZE);

        let mut code = decode(bytes.iter().map(|i| *i)).collect::<Vec<Instruction>>();
        let mut program = [0; PROGRAM_SIZE];
        bytes.resize(PROGRAM_SIZE, 0);
        program.copy_from_slice(&bytes);

        let mut code_array = [NOP; MAX_INSTRUCTIONS];
        code.resize(MAX_INSTRUCTIONS, NOP);
        code_array.copy_from_slice(&code);

        Memory {
            code: code_array,
            data: [0; SRAM_SIZE],
            program: program,
            io: io,
            ports: [Port::new(io, 0), Port::new(io, 1), Port::new(io, 2), Port::new(io, 3)],
        }
    }

    #[inline(always)]
    pub fn get_instruction(&self, ip: usize) -> Instruction {
        self.code[ip]
    }

    #[inline(always)]
    pub fn reg(&self, index: u8) -> u8 {
        debug_assert!(index < NUM_REGISTER);
        // we don't need to call data, because ports
        // should only change io register, not normal ones
        self.data[(REGISTER_OFFSET + index) as usize]
    }

    // TODO change to set_reg instead of returning a reference
    #[inline(always)]
    pub fn reg_mut(&mut self, index: u8) -> &mut u8 {
        debug_assert!(index < NUM_REGISTER);
        &mut self.data[(REGISTER_OFFSET + index) as usize]
    }

    #[inline(always)]
    pub fn io_reg(&self, index: u8) -> u8 {
        debug_assert!(index < NUM_IO_REGISTER);
        self.data((IO_REGISTER_OFFSET + index) as u16)
    }

    #[inline(always)]
    pub fn set_io_reg(&mut self, index: u8, val: u8) {
        debug_assert!(index < NUM_IO_REGISTER);
        self.set_data((IO_REGISTER_OFFSET + index) as u16, val);
    }

    #[inline(always)]
    pub fn read_program(&self, index: u16) -> u8 {
        self.program[index as usize]
    }

    #[inline(always)]
    pub fn flags(&self) -> u8 {
        // don't call data because flags is not used
        // by the ports and we can skip the checks
        self.data[(IO_REGISTER_OFFSET + FLAGS_REG) as usize]
    }

    #[inline(always)]
    pub fn set_flags(&mut self, flags: u8) {
        // don't call data because flags is not used
        // by the ports and we can skip the checks
        self.data[(IO_REGISTER_OFFSET + FLAGS_REG) as usize] = flags;
    }

    #[inline(always)]
    pub fn data(&self, index: u16) -> u8 {
        for port in self.ports.iter() {
            if let Some(ret) = port.read(index) {
                return ret;
            }
        }

        self.data[index as usize]
    }

    #[inline(always)]
    pub fn set_data(&mut self, index: u16, val: u8) {
        self.data[index as usize] = val;

        if index == UDR {
            #[cfg(debug_assertions)]
            println!("Output: {}", char::from_u32(val as u32).unwrap_or('?'));
            #[cfg(not(debug_assertions))]
            print!("{}", char::from_u32(val as u32).unwrap_or('?'));
        }

        if let Some(io) = self.io {
            adc_write(io, &mut self.data, index, val);
        }

        for port in self.ports.iter_mut() {
            port.write(index, val);
        }
    }

    #[inline(always)]
    pub fn io_reg16(&self, index: u8) -> u16 {
        ((self.io_reg(index + 1) as u16) << 8) | (self.io_reg(index) as u16)
    }

    #[inline(always)]
    pub fn set_io_reg16(&mut self, index: u8, val: u16) {
        self.set_io_reg(index, val as u8);
        self.set_io_reg(index + 1, (val >> 8) as u8);
    }

    #[inline(always)]
    pub fn sp(&self) -> u16 {
        self.io_reg16(SP_REG)
    }

    #[inline(always)]
    pub fn set_sp(&mut self, val: u16) {
        self.set_io_reg16(SP_REG, val);
    }

    #[inline(always)]
    pub fn push(&mut self, val: u8) {
        let sp = self.sp();
        self.set_data(sp, val);
        self.set_sp(sp.wrapping_sub(1));
    }

    #[inline(always)]
    pub fn pop(&mut self) -> u8 {
        let sp = self.sp().wrapping_add(1);

        let ret = self.data(sp);
        self.set_sp(sp);
        ret
    }

    #[inline(always)]
    pub fn push16(&mut self, val: u16) {
        self.push(val as u8);
        self.push((val >> 8) as u8);
    }

    #[inline(always)]
    pub fn pop16(&mut self) -> u16 {
        let top = self.pop() as u16;
        let bot = self.pop() as u16;
        (top << 8) | bot
    }
}
