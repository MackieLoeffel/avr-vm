use data;
use data::{Instruction, LDType, LDMode, LPMType};
use data::Instruction::*;
use memory::{Memory};
use interrupts::{PortInterrupts, TimerInterrupts};
use util::{bit, bit16, bitneg, bitneg16};
#[cfg(feature = "jit")]
use std::collections::HashMap;
#[cfg(feature = "jit")]
use std::{mem};
#[cfg(feature = "jit")]
use data::{Register, SREG};

#[cfg(feature = "jit")]
use dynasmrt;
#[cfg(feature = "jit")]
use dynasmrt::{ExecutableBuffer, AssemblyOffset};
#[cfg(feature = "jit")]
use dynasmrt::DynasmApi;

const I: usize = 7;
const T: usize = 6;
const H: usize = 5;
const S: usize = 4;
const V: usize = 3;
const N: usize = 2;
const Z: usize = 1;
const C: usize = 0;

pub struct Cpu<'a> {
    ip: usize,
    mem: Memory<'a>,
    #[allow(dead_code)]
    steps: u32,
    // needed for tests and timing
    halt_on_nop: bool,
     // cpu should should stop forever, for halt on nop
    should_halt: bool,
    sleeping: bool,
    port_int: PortInterrupts,
    timer_int: TimerInterrupts,
    // we can't just save the function pointer, because then we would free
    // the buffer and segfault, when we try to execute the function
    #[cfg(feature = "jit")]
    blocks: HashMap<usize, (ExecutableBuffer, AssemblyOffset)>
}

impl<'a> Cpu<'a> {
    pub fn new(mem: Memory, halt_on_nop: bool) -> Cpu {
        #[cfg(not(feature = "jit"))]
        {Cpu { ip: 0, mem: mem, steps: 0,
              halt_on_nop: halt_on_nop, sleeping: false, should_halt: false,
              port_int: PortInterrupts::new(), timer_int: TimerInterrupts::new(),
        }}
        #[cfg(feature = "jit")]
        {Cpu { ip: 0, mem: mem, steps: 0,
              halt_on_nop: halt_on_nop, sleeping: false, should_halt: false,
              port_int: PortInterrupts::new(), timer_int: TimerInterrupts::new(),
              blocks: HashMap::new()
        }}
    }

    pub fn step(&mut self) -> bool {
        if self.should_halt {
            return false;
        }

        #[cfg(debug_assertions)]
        {
            if self.steps > 10000 {
                panic!("Maximum number of steps reached!");
            }
            self.steps += 1;
        }

        self.port_int.step(&mut self.mem);
        self.timer_int.step(&mut self.mem);

        if bit(self.flags(), I) == 1 {
            if let Some(interrupt_nr) = self.pending_interrupt() {
                self.sleeping = false;
                self.set_flags(Some(0), None, None, None, None, None, None);
                self.mem.push16(self.ip as u16);
                self.ip = interrupt_nr << 1; // jump to the interrupt
            }
        }

        if !self.sleeping {

            #[cfg(not(feature = "jit"))]
            {
                let instr = self.mem.get_instruction(self.ip);
                self.handle_instruction(instr);
            }

            #[cfg(feature = "jit")]
            {
                let func: extern "sysv64" fn(*mut Cpu);
                {
                    let mem = &self.mem;
                    let ip = self.ip;
                    #[cfg(debug_assertions)]
                    println!("Executing {}", self.ip);
                    let entry = self.blocks.entry(ip).or_insert_with(|| Cpu::compile_block(mem, ip));
                    func = unsafe {
                        mem::transmute(entry.0.ptr(entry.1))
                    };
                }
                func(self);
            }
        }

        true
    }

    #[inline(always)]
    fn pending_interrupt(&mut self) -> Option<usize> {
        // we must handle the interrupt, if one of the two interrupt sources
        // returns Some. So we must use .or_else and can't use .or
        self.port_int.pending_interrupt(&mut self.mem).or_else(|| {
            self.timer_int.pending_interrupt(&mut self.mem)
        })
    }

    #[cfg(feature = "jit")]
    #[inline(always)]
    fn compile_block(mem: &Memory, addr: usize) -> (ExecutableBuffer, AssemblyOffset) {
        #[cfg(debug_assertions)]
        print!("Compiling {}", addr);
        // we use standard C calling convention, which is documented here:
        // https://en.wikipedia.org/wiki/X86_calling_conventions#System_V_AMD64_ABI
        // this is the standard calling x64 convention on linux

        macro_rules! cpu_call(
            ($ops:expr, $func:expr) => {
                dynasm!($ops
                        ; mov rdi, r12
                        ; mov rax, QWORD $func as _
                        ; call rax)
            };
            ($ops:expr, $func:expr, $arg1:expr) => {
                dynasm!($ops
                        ; mov rsi, QWORD $arg1 as _
                        ;; cpu_call!($ops, $func))
            };
            ($ops:expr, $func:expr, $arg1:expr, $arg2:expr) => {
                dynasm!($ops
                        ; mov rdx, QWORD $arg2 as _
                        ;; cpu_call!($ops, $func, $arg1))
            };
            ($ops:expr, $func:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
                dynasm!($ops
                        ; mov rcx, QWORD $arg3 as _
                        ;; cpu_call!($ops, $func, $arg1, $arg2))
            };
            ($ops:expr, $func:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
                dynasm!($ops
                        ; mov r8, QWORD $arg4 as _
                        ;; cpu_call!($ops, $func, $arg1, $arg2, $arg3))
            };
            ($ops:expr, $func:expr, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {
                dynasm!($ops
                        ; mov r9, QWORD $arg5 as _
                        ;; cpu_call!($ops, $func, $arg1, $arg2, $arg3, $arg4))
            };
        );

        let mut ops = dynasmrt::Assembler::new();

        let offset = ops.offset();
        dynasm!(ops
                ; push r12       // r12 is callee-save
                ; mov r12, rdi); // save cpu pointer in r12

        let mut cur_addr = addr;
        loop {
            let instr = mem.get_instruction(cur_addr);
            #[cfg(debug_assertions)]
            print!(" {:?} ", instr);

            match instr {
                SecondOpWord => {}
                ADD(rd, rr) => cpu_call!(ops, add, rd, rr),
                ADC(rd, rr) => cpu_call!(ops, adc, rd, rr),
                ADIW(reg, k) => cpu_call!(ops, adiw, reg, k),
                AND(rd, rr) => cpu_call!(ops, and, rd, rr),
                ANDI(rd, k) => cpu_call!(ops, andi, rd, k),
                ASR(reg) => cpu_call!(ops, asr, reg),
                BCLR(s) => cpu_call!(ops, bclr, s),
                BLD_ST(LDType::LD, reg, b) => cpu_call!(ops, bld, reg, b),
                BLD_ST(LDType::ST, reg, b) => cpu_call!(ops, bst, reg, b),
                BSET(s) => cpu_call!(ops, bset, s),
                BRBC_S(sc, sreg, rel) => cpu_call!(ops, brbc_s, sc.as_u8(), sreg, rel),
                CALL(ip) => cpu_call!(ops, call, ip),
                COM(reg) => cpu_call!(ops, com, reg),
                CP(rd, rr) => cpu_call!(ops, cp, rd, rr),
                CPC(rd, rr) => cpu_call!(ops, cpc, rd, rr),
                CPI(reg, k) => cpu_call!(ops, cpi, reg, k),
                CPSE(rd, rr) => cpu_call!(ops, cpse, rd, rr),
                C_SBI(typ, ioreg, b) => cpu_call!(ops, c_sbi, typ.as_u8(), ioreg, b),
                DEC(reg) => cpu_call!(ops, dec, reg),
                EOR(rd, rr) => cpu_call!(ops, eor, rd, rr),
                ICALL => cpu_call!(ops, icall),
                IN(reg, index) => cpu_call!(ops, in_, reg, index),
                JMP(ip) => cpu_call!(ops, jmp, ip),
                LD_ST(typ, reg, addrreg, mode) => cpu_call!(ops, ld_st, typ.as_u8(), reg, addrreg, mode.as_u16()),
                LD_STS(LDType::LD, reg, k) => cpu_call!(ops, lds, reg, k),
                LD_STS(LDType::ST, reg, k) => cpu_call!(ops, sts, reg, k),
                LDI(reg, val) => cpu_call!(ops, ldi, reg, val),
                LPM(reg, typ) => cpu_call!(ops, lpm, reg, typ.as_u8()),
                LSR(reg) => cpu_call!(ops, lsr, reg),
                MOV(rd, rr) => cpu_call!(ops, mov, rd, rr),
                MOVW(rd, rr) => cpu_call!(ops, movw, rd, rr),
                MUL(rd, rr) => cpu_call!(ops, mul, rd, rr),
                NEG(rd) => cpu_call!(ops, neg, rd),
                NOP => cpu_call!(ops, nop),
                OR(rd, rv) => cpu_call!(ops, or, rd, rv),
                ORI(rd, k) => cpu_call!(ops, ori, rd, k),
                OUT(reg, index) => cpu_call!(ops, out, reg, index),
                POP(reg) => cpu_call!(ops, pop, reg),
                PUSH(reg) => cpu_call!(ops, push, reg),
                RCALL(diff) => cpu_call!(ops, rcall, diff),
                RET => cpu_call!(ops, ret),
                RETI => cpu_call!(ops, reti),
                RJMP(diff) => cpu_call!(ops, rjmp, diff),
                ROR(reg) => cpu_call!(ops, ror, reg),
                SBCI(reg, val) => cpu_call!(ops, sbci, reg, val),
                SBC(rd, rr) => cpu_call!(ops, sbc, rd, rr),
                SBIW(reg, k) => cpu_call!(ops, sbiw, reg, k),
                SBIC_S(setclear, reg, b) => cpu_call!(ops, sbic_s, setclear.as_u8(), reg, b),
                SBR(setclear, reg, b) => cpu_call!(ops, sbr, setclear.as_u8(), reg, b),
                SLEEP => cpu_call!(ops, sleep),
                SUB(rd, rr) => cpu_call!(ops, sub, rd, rr),
                SUBI(reg, val) => cpu_call!(ops, subi, reg, val),
                i@_ => panic!("ip: {:#x}, Unknown Instruction: {:?}", cur_addr, i)
            }

            if Cpu::is_end_of_block(instr) {
                break;
            }
            cur_addr += 1;
        }
        dynasm!(ops
                ; pop r12
                ; ret
        );
        #[cfg(debug_assertions)]
        println!("");
        let buf = ops.finalize().unwrap();
        (buf, offset)
    }

    #[inline(always)]
    #[cfg(feature = "jit")]
    fn is_end_of_block(instr: Instruction) -> bool {
        // we can't enable interrupts directly after sti but need to enable them one step later
        // the delay is needed e.g. for a sti sleep sequence, where we must not miss
        // an interrupt between the two or we may never wakeup
        // BSET(I) must never be at the end of a block!

        match instr {
            BRBC_S(..) | CALL(..) | CPSE(..) | ICALL | JMP(..) | NOP
                | RCALL(..) | RET | RETI | RJMP(..)
                | SBIC_S(..) | SBR(..) | SLEEP => true,
            _ => false
        }
    }


    #[inline(always)]
    #[cfg(not(feature = "jit"))]
    fn handle_instruction(&mut self, instr: Instruction) {
        #[cfg(debug_assertions)]
        println!("Executing: {:?}", instr);
        match instr {
            ADD(rd, rr) => {
                let rdv = self.reg(rd);
                let rrv = self.reg(rr);
                let res = rdv.wrapping_add(rrv);
                *self.reg_mut(rd) = res;
                self.set_flags(
                    None, None,
                    Some(bit(rdv, 3) & bit(rrv, 3)
                         | bitneg(res, 3) & (bit(rdv, 3) | bit(rrv, 3))),
                    Some(bit(rdv, 7) & bit(rrv, 7) & bitneg(res, 7)
                         | bitneg(rdv, 7) & bitneg(rrv, 7) & bit(res, 7)),
                    Some(bit(res, 7)),
                    Some((res == 0) as u8),
                    Some(bit(rdv, 7) & bit(rrv, 7)
                         | bitneg(res, 7) & (bit(rdv, 7) | bit(rrv, 7))));
                self.ip += 1;
            },
            ADC(rd, rr) => {
                let rdv = self.reg(rd);
                let rrv = self.reg(rr);
                let res = rdv.wrapping_add(rrv).wrapping_add(bit(self.flags(), C));
                *self.reg_mut(rd) = res;

                self.set_flags(
                    None, None,
                    Some(bit(rdv, 3) & bit(rrv, 3)
                         | bitneg(res, 3) & (bit(rdv, 3) | bit(rrv, 3))),
                    Some(bit(rdv, 7) & bit(rrv, 7) & bitneg(res, 7)
                         | bitneg(rdv, 7) & bitneg(rrv, 7) & bit(res, 7)),
                    Some(bit(res, 7)),
                    Some((res == 0) as u8),
                    Some(bit(rdv, 7) & bit(rrv, 7)
                         | bitneg(res, 7) & (bit(rdv, 7) | bit(rrv, 7))));
                self.ip += 1;
            },
            ADIW(reg, k) => {
                let rdv = self.get_word_reg(reg);
                let res = rdv.wrapping_add(k as u16);
                self.set_word_reg(reg, res);

                self.set_flags(None, None, None,
                               Some(bitneg16(rdv, 15) & bit16(res, 15)),
                               Some(bit16(res, 15)), Some((res == 0) as u8),
                               Some(bit16(rdv, 15) & bitneg16(res, 15)));
                self.ip += 1;
            }
            AND(rd, rr) => {
                let res = self.reg(rd) & self.reg(rr);
                *self.reg_mut(rd) = res;
                self.set_flags(
                    None, None, None, Some(0),
                    Some(res >> 7), Some((res == 0) as u8), None);

                self.ip += 1;
            },
            ANDI(rd, k) => {
                let res = self.reg(rd) & k;
                *self.reg_mut(rd) = res;
                self.set_flags(
                    None, None, None, Some(0),
                    Some(res >> 7), Some((res == 0) as u8), None);

                self.ip += 1;
            },
            ASR(reg) => {
                let rdv = self.reg(reg);
                let res = ((rdv as i8) >> 1) as u8;
                *self.reg_mut(reg) = res;
                self.set_flags(None, None, None, Some(bit(res, 7) ^ bit(rdv, 0)),
                               Some(bit(res, 7)), Some((res == 0) as u8),
                               Some(bit(rdv, 0)));
                self.ip += 1;
            }
            BCLR(s) => {
                let flags = self.flags();
                self.mem.set_flags(flags & !(1 << s));
                self.ip += 1;
            },
            BLD_ST(LDType::LD, reg, b) => {
                let rdv = self.reg(reg);
                let t = bit(self.flags(), T);
                *self.reg_mut(reg) = rdv & !(1 << b) | (t << b);
                self.ip += 1;
            }
            BLD_ST(LDType::ST, reg, b) => {
                let rdv = self.reg(reg);
                self.set_flags(None, Some(bit(rdv, b as usize)), None, None, None, None, None);
                self.ip += 1;
            }
            BSET(s) => {
                let flags = self.flags();
                self.mem.set_flags(flags | (1 << s));
                self.ip += 1;
            }
            BRBC_S(typ, sreg, rel) => {
                if bit(self.flags(), sreg as usize) == typ.as_u8() {
                    self.ip = (self.ip as i32 + rel as i32) as usize;
                } else {
                    self.ip += 1;
                }
            },
            CALL(ip) => {
                let retip = (self.ip + 2) as u16;
                self.mem.push16(retip);

                self.ip = ip as usize;
            }
            COM(reg) => {
                let res = !self.reg(reg);
                *self.reg_mut(reg) = res;
                self.set_flags(None, None, None, Some(0), Some(bit(res, 7)), Some((res == 0) as u8), Some(1));
                self.ip += 1;
            }
            CP(rd, rr) => {
                let rdv = self.reg(rd);
                let rrv = self.reg(rr);
                let res = rdv.wrapping_sub(rrv);

                self.set_flags(None, None,
                               Some(bitneg(rdv, 3) & bit(rrv, 3)
                                    | bit(rrv, 3) & bit(res, 3)
                                    | bit(res, 3) & bitneg(rdv, 3)),
                               Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                                    | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8),
                               Some(bitneg(rdv, 7) & bit(rrv, 7)
                                    | bit(rrv, 7) & bit(res, 7)
                                    | bit(res, 7) & bitneg(rdv, 7)));

                self.ip += 1;
            }
            CPC(rd, rr) => {
                let rdv = self.reg(rd);
                let rrv = self.reg(rr);
                let res = rdv.wrapping_sub(rrv).wrapping_sub(bit(self.flags(), C));

                let z = bit(self.flags(), Z);
                self.set_flags(None, None,
                               Some(bitneg(rdv, 3) & bit(rrv, 3)
                                    | bit(rrv, 3) & bit(res, 3)
                                    | bit(res, 3) & bitneg(rdv, 3)),
                               Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                                    | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8 & z),
                               Some(bitneg(rdv, 7) & bit(rrv, 7)
                                    | bit(rrv, 7) & bit(res, 7)
                                    | bit(res, 7) & bitneg(rdv, 7)));

                self.ip += 1;
            }
            CPI(reg, k) => {
                let rv = self.reg(reg);
                let res = rv.wrapping_sub(k);

                self.set_flags(None, None, Some(bitneg(rv, 3) & bit(k, 3)
                               | bit(k, 3) & bit(res, 3) | bit(res, 3) & bitneg(rv, 3)),
                               Some(bit(rv, 7) & bitneg(k, 7) & bitneg(res, 7)
                                    | bitneg(rv, 7) & bit(k, 7) & bit(res, 7)),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8),
                               Some(bitneg(rv, 7) & bit(k, 7)
                                    | bit(k, 7) & bit(res, 7)
                                    | bit(res, 7) & bitneg(rv, 7)));

                self.ip += 1;
            },
            CPSE(rd, rr) => {
                if self.reg(rd) == self.reg(rr) {
                    self.ip += 2;
                    if self.mem.get_instruction(self.ip) == SecondOpWord {
                        self.ip += 1;
                    }
                } else {
                    self.ip += 1;
                }
            }
            C_SBI(typ, ioreg, b) => {
                let val = self.mem.io_reg(ioreg);
                self.mem.set_io_reg(ioreg, (val & !(1 << b)) | (typ.as_u8() << b));
                self.ip += 1;
            }
            DEC(reg) => {
                let res = self.reg(reg).wrapping_sub(1);
                *self.reg_mut(reg) = res;
                self.set_flags(None, None, None, Some((res == 0x7f) as u8),
                               Some(bit(res, 7)), Some((res == 0) as u8), None);
                self.ip += 1;
            }
            EOR(rd, rr) => {
                let res = self.reg(rd) ^ self.reg(rr);
                *self.reg_mut(rd) = res;
                self.set_flags(None, None, None, Some(0), Some(bit(res, 7)),
                               Some((res == 0) as u8), None);
                self.ip += 1;
            },
            ICALL => {
                let retip = (self.ip + 1) as u16;
                self.mem.push16(retip);

                let ip = self.get_word_reg(data::Z);
                self.ip = ip as usize;
            },
            IN(reg, index) => {
                *self.reg_mut(reg) = self.mem.io_reg(index);
                self.ip += 1;
            }
            JMP(ip) => self.ip = ip as usize,
            LD_ST(typ, reg, addrreg, mode) => {
                let displacement = match mode {
                    LDMode::PreDecrement => {
                        let modval = self.get_word_reg(addrreg).wrapping_sub(1);
                        self.set_word_reg(addrreg, modval);
                        0
                    },
                    LDMode::PostIncrement => 0,
                    LDMode::Displacement(i) => i
                };

                let addr = self.get_word_reg(addrreg).wrapping_add(displacement as u16);
                match typ {
                    LDType::LD => {
                        *self.reg_mut(reg) = self.mem.data(addr);
                    },
                    LDType::ST => {
                        let val = self.reg(reg);
                        self.mem.set_data(addr, val);
                    }
                }

                match mode {
                    LDMode::PostIncrement => {
                        let modval = self.get_word_reg(addrreg).wrapping_add(1);
                        self.set_word_reg(addrreg, modval);
                    },
                    _ => {}
                };
                self.ip += 1;
            },
            LD_STS(LDType::LD, reg, k) => {
                let val = self.mem.data(k);
                *self.reg_mut(reg) = val;
                self.ip += 2;
            }
            LD_STS(LDType::ST, reg, k) => {
                let val = self.reg(reg);
                self.mem.set_data(k, val);
                self.ip += 2;
            }
            LDI(reg, val) => {
                *self.reg_mut(reg) = val;
                self.ip += 1;
            },
            LPM(reg, typ) => {
                let z = self.get_word_reg(data::Z);
                *self.reg_mut(reg) = self.mem.read_program(z);

                match typ {
                    LPMType::ZPostIncrement => self.set_word_reg(data::Z, z.wrapping_add(1)),
                    _ => {}
                }

                self.ip += 1;
            }
            LSR(reg) => {
                let rdv = self.reg(reg);
                let res = rdv >> 1;
                *self.reg_mut(reg) = res;
                self.set_flags(None, None, None, Some(bit(rdv, 0)),
                               Some(0), Some((res == 0) as u8),
                               Some(bit(rdv, 0)));
                self.ip += 1;
            }
            MOV(rd, rr) => {
                *self.reg_mut(rd) = self.reg(rr);
                self.ip += 1;
            }
            MOVW(rd, rr) => {
                let val = self.get_word_reg(rr);
                self.set_word_reg(rd, val);
                self.ip += 1;
            },
            MUL(rd, rr) => {
                let res = (self.reg(rd) as u16) * (self.reg(rr) as u16);
                self.set_word_reg(0, res);
                self.set_flags(None, None, None, None, None, Some((res == 0) as u8), Some(bit16(res, 15)));
                self.ip += 1;
            }
            NEG(rd) => {
                let rdv = self.reg(rd);
                let res = (0 as i8).wrapping_sub(rdv as i8) as u8;
                *self.reg_mut(rd) = res;
                self.set_flags(None, None, Some(bit(res, 3) | bitneg(rdv, 3)),
                               Some((res == 0x80) as u8),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8),
                               Some((res != 0) as u8));
                self.ip += 1;
            }
            NOP => {
                if self.halt_on_nop {
                    self.should_halt = true;
                }

                self.ip += 1;
            }
            OR(rd, rv) => {
                let res = self.reg(rd) | self.reg(rv);
                *self.reg_mut(rd) = res;
                self.set_flags(None, None, None, Some(0), Some(bit(res, 7)), Some((res == 0) as u8), None);
                self.ip += 1;
            }
            ORI(rd, k) => {
                let res = self.reg(rd) | k;
                *self.reg_mut(rd) = res;
                self.set_flags(None, None, None, Some(0), Some(bit(res, 7)), Some((res == 0) as u8), None);
                self.ip += 1;
            }
            OUT(reg, index) => {
                let val = self.reg(reg);
                self.mem.set_io_reg(index, val);
                self.ip += 1;
            },
            POP(reg) => {
                let val = self.mem.pop();
                *self.reg_mut(reg) = val;

                self.ip += 1;
            },
            PUSH(reg) => {
                let val = self.reg(reg);
                self.mem.push(val);

                self.ip += 1;
            }
            RCALL(diff) => {
                let retip = (self.ip + 1) as u16;
                self.mem.push16(retip);

                self.ip = (self.ip as i32 + diff as i32) as usize;
            },
            RET => {
                self.ip = self.mem.pop16() as usize;
            }
            RETI => {
                self.set_flags(Some(1), None, None, None, None, None, None);
                self.ip = self.mem.pop16() as usize;
            }
            RJMP(diff) => {
                self.ip = (self.ip as i32 + diff as i32) as usize;
            },
            ROR(reg) => {
                let rdv = self.reg(reg);
                let res = bit(self.flags(), C) << 7 | (rdv >> 1);
                *self.reg_mut(reg) = res;
                self.set_flags(None, None, None, Some(bit(res, 7) ^ bit(rdv, 0)),
                               Some(bit(res, 7)), Some((res == 0) as u8),
                               Some(bit(rdv, 0)));
                self.ip += 1;
            }
            SBCI(reg, val) => {
                let rdv = self.reg(reg);
                let rrv = val;
                let res = rdv.wrapping_sub(rrv).wrapping_sub(bit(self.flags(), C));
                *self.reg_mut(reg) = res;

                let z = bit(self.flags(), Z);
                self.set_flags(None, None,
                               Some(bitneg(rdv, 3) & bit(rrv, 3)
                                    | bit(rrv, 3) & bit(res, 3)
                                    | bit(res, 3) & bitneg(rdv, 3)),
                               Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                                    | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8 & z),
                               Some(bitneg(rdv, 7) & bit(rrv, 7)
                                    | bit(rrv, 7) & bit(res, 7)
                                    | bit(res, 7) & bitneg(rdv, 7)));

                self.ip += 1;
            },
            SBC(rd, rr) => {
                let rdv = self.reg(rd);
                let rrv = self.reg(rr);
                let res = rdv.wrapping_sub(rrv).wrapping_sub(bit(self.flags(), C));
                *self.reg_mut(rd) = res;

                let z = bit(self.flags(), Z);
                self.set_flags(None, None,
                               Some(bitneg(rdv, 3) & bit(rrv, 3)
                                    | bit(rrv, 3) & bit(res, 3)
                                    | bit(res, 3) & bitneg(rdv, 3)),
                               Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                                    | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8 & z),
                               Some(bitneg(rdv, 7) & bit(rrv, 7)
                                    | bit(rrv, 7) & bit(res, 7)
                                    | bit(res, 7) & bitneg(rdv, 7)));

                self.ip += 1;
            },
            SBIW(reg, k) => {
                let rdv = self.get_word_reg(reg);
                let res = rdv.wrapping_sub(k as u16);
                self.set_word_reg(reg, res);

                self.set_flags(None, None, None,
                               Some(bit16(rdv, 15) & bitneg16(res, 15)),
                               Some(bit16(res, 15)), Some((res == 0) as u8),
                               Some(bitneg16(rdv, 15) & bit16(res, 15)));
                self.ip += 1;
            },
            SBIC_S(setclear, reg, b) => {
                if bit(self.mem.io_reg(reg), b as usize) == setclear.as_u8() {
                    self.ip += 2;
                    if self.mem.get_instruction(self.ip) == SecondOpWord {
                        self.ip += 1;
                    }
                } else {
                    self.ip += 1;
                }
            }
            SBR(setclear, reg, b) => {
                if bit(self.reg(reg), b as usize) == setclear.as_u8() {
                    self.ip += 2;
                    if self.mem.get_instruction(self.ip) == SecondOpWord {
                        self.ip += 1;
                    }
                } else {
                    self.ip += 1;
                }
            }
            SLEEP => {
                self.sleeping = true;
                self.ip += 1;
            }
            SUB(rd, rr) => {
                let rdv = self.reg(rd);
                let rrv = self.reg(rr);
                let res = rdv.wrapping_sub(rrv);
                *self.reg_mut(rd) = res;
                self.set_flags(None, None,
                               Some(bitneg(rdv, 3) & bit(rrv, 3)
                                    | bit(rrv, 3) & bit(res, 3)
                                    | bit(res, 3) & bitneg(rdv, 3)),
                               Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                                    | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8),
                               Some(bitneg(rdv, 7) & bit(rrv, 7)
                                    | bit(rrv, 7) & bit(res, 7)
                                    | bit(res, 7) & bitneg(rdv, 7)));

                self.ip += 1;
            }
            SUBI(reg, val) => {
                let rdv = self.reg(reg);
                let rrv = val;
                let res = rdv.wrapping_sub(rrv);
                *self.reg_mut(reg) = res;
                self.set_flags(None, None,
                               Some(bitneg(rdv, 3) & bit(rrv, 3)
                                    | bit(rrv, 3) & bit(res, 3)
                                    | bit(res, 3) & bitneg(rdv, 3)),
                               Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                                    | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                               Some(bit(res, 7)),
                               Some((res == 0) as u8),
                               Some(bitneg(rdv, 7) & bit(rrv, 7)
                                    | bit(rrv, 7) & bit(res, 7)
                                    | bit(res, 7) & bitneg(rdv, 7)));

                self.ip += 1;
            }
            i@_ => panic!("ip: {:#x}, Unknown Instruction: {:?}", self.ip << 1, i)
        }
    }

    // sets the flags
    // if a flag is None, it isn't changed, otherwise it is set to its value
    // the value (in the Some) must be 0 or 1
    // s is calculated
    #[inline(always)]
    fn set_flags(&mut self, i: Option<u8>, t: Option<u8>, h: Option<u8>, v: Option<u8>, n: Option<u8>, z: Option<u8>, c: Option<u8>) {
        let set = (i.is_some() as u8) << I
            | (t.is_some() as u8) << T
            | (h.is_some() as u8) << H
            // s is calculated
            | (v.is_some() as u8) << V
            | (n.is_some() as u8) << N
            | (z.is_some() as u8) << Z
            | (c.is_some() as u8) << C;

        let val = (i.unwrap_or(0) as u8) << I
            | (t.unwrap_or(0) as u8) << T
            | (h.unwrap_or(0) as u8) << H
            // s is calculated
            | (v.unwrap_or(0) as u8) << V
            | (n.unwrap_or(0) as u8) << N
            | (z.unwrap_or(0) as u8) << Z
            | (c.unwrap_or(0) as u8) << C;

        let mut flags = self.flags();
        flags &= !set;
        flags |= val;

        // calculate s
        flags &= !((1 as u8) << S);
        flags |= (bit(flags, N) ^ bit(flags, V)) << S;
        self.mem.set_flags(flags);
    }

    #[inline(always)]
    fn get_word_reg(&self, reg: u8) -> u16 {
        assert_eq!(reg % 2, 0, "reg must be even");
        (self.reg(reg + 1) as u16) << 8
            | (self.reg(reg) as u16)
    }

    #[inline(always)]
    fn set_word_reg(&mut self, reg: u8, val: u16) {
        assert_eq!(reg % 2, 0, "reg must be even");
        *self.reg_mut(reg) = val as u8;
        *self.reg_mut(reg + 1) = (val >> 8) as u8;
    }

    #[inline(always)]
    fn reg_mut(&mut self, index: u8) -> &mut u8 {
        self.mem.reg_mut(index)
    }

    #[inline(always)]
    fn reg(&self, index: u8) -> u8 {
        self.mem.reg(index)
    }

    #[inline(always)]
    fn flags(&self) -> u8 {
        self.mem.flags()
    }
}

#[cfg(feature = "jit")]
extern "sysv64" fn add(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(rd);
    let rrv = cpu.reg(rr);
    let res = rdv.wrapping_add(rrv);
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(
        None, None,
        Some(bit(rdv, 3) & bit(rrv, 3)
             | bitneg(res, 3) & (bit(rdv, 3) | bit(rrv, 3))),
        Some(bit(rdv, 7) & bit(rrv, 7) & bitneg(res, 7)
             | bitneg(rdv, 7) & bitneg(rrv, 7) & bit(res, 7)),
        Some(bit(res, 7)),
        Some((res == 0) as u8),
        Some(bit(rdv, 7) & bit(rrv, 7)
             | bitneg(res, 7) & (bit(rdv, 7) | bit(rrv, 7))));
    cpu.ip += 1;
}

#[cfg(feature = "jit")]
extern "sysv64" fn adc(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(rd);
    let rrv = cpu.reg(rr);
    let res = rdv.wrapping_add(rrv).wrapping_add(bit(cpu.flags(), C));
    *cpu.reg_mut(rd) = res;

    cpu.set_flags(
        None, None,
        Some(bit(rdv, 3) & bit(rrv, 3)
             | bitneg(res, 3) & (bit(rdv, 3) | bit(rrv, 3))),
        Some(bit(rdv, 7) & bit(rrv, 7) & bitneg(res, 7)
                         | bitneg(rdv, 7) & bitneg(rrv, 7) & bit(res, 7)),
        Some(bit(res, 7)),
        Some((res == 0) as u8),
        Some(bit(rdv, 7) & bit(rrv, 7)
             | bitneg(res, 7) & (bit(rdv, 7) | bit(rrv, 7))));
    cpu.ip += 1;
}

#[cfg(feature = "jit")]
extern "sysv64" fn adiw(c: *mut Cpu, reg: Register, k: u8) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.get_word_reg(reg);
    let res = rdv.wrapping_add(k as u16);
    cpu.set_word_reg(reg, res);

    cpu.set_flags(None, None, None,
                   Some(bitneg16(rdv, 15) & bit16(res, 15)),
                   Some(bit16(res, 15)), Some((res == 0) as u8),
                   Some(bit16(rdv, 15) & bitneg16(res, 15)));
    cpu.ip += 1;
}

#[cfg(feature = "jit")]
extern "sysv64" fn and(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let res = cpu.reg(rd) & cpu.reg(rr);
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(
        None, None, None, Some(0),
        Some(res >> 7), Some((res == 0) as u8), None);

    cpu.ip += 1;
}

#[cfg(feature = "jit")]
extern "sysv64" fn andi(c: *mut Cpu, rd: Register, k: u8) {
    let cpu = unsafe {&mut *c};
    let res = cpu.reg(rd) & k;
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(
        None, None, None, Some(0),
        Some(res >> 7), Some((res == 0) as u8), None);

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn asr(c: *mut Cpu, reg: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(reg);
    let res = ((rdv as i8) >> 1) as u8;
    *cpu.reg_mut(reg) = res;
    cpu.set_flags(None, None, None, Some(bit(res, 7) ^ bit(rdv, 0)),
                   Some(bit(res, 7)), Some((res == 0) as u8),
                   Some(bit(rdv, 0)));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn bclr(c: *mut Cpu, s: SREG) {
    let cpu = unsafe {&mut *c};
    let flags = cpu.flags();
    cpu.mem.set_flags(flags & !(1 << s));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn bld(c: *mut Cpu, reg: Register, b: u8) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(reg);
    let t = bit(cpu.flags(), T);
    *cpu.reg_mut(reg) = rdv & !(1 << b) | (t << b);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn bst(c: *mut Cpu, reg: Register, b: u8) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(reg);
    cpu.set_flags(None, Some(bit(rdv, b as usize)), None, None, None, None, None);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn bset(c: *mut Cpu, s: SREG) {
    let cpu = unsafe {&mut *c};
    let flags = cpu.flags();
    cpu.mem.set_flags(flags | (1 << s));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn brbc_s(c: *mut Cpu, setclear: u8, sreg: SREG, rel: i8) {
    let cpu = unsafe {&mut *c};
    if bit(cpu.flags(), sreg as usize) == setclear {
        cpu.ip = (cpu.ip as i32 + rel as i32) as usize;
    } else {
        cpu.ip += 1;
    }
}
#[cfg(feature = "jit")]
extern "sysv64" fn call(c: *mut Cpu, ip: u32) {
    let cpu = unsafe {&mut *c};
    let retip = (cpu.ip + 2) as u16;
    cpu.mem.push16(retip);

    cpu.ip = ip as usize;
}
#[cfg(feature = "jit")]
extern "sysv64" fn com(c: *mut Cpu, reg: Register) {
    let cpu = unsafe {&mut *c};
    let res = !cpu.reg(reg);
    *cpu.reg_mut(reg) = res;
    cpu.set_flags(None, None, None, Some(0), Some(bit(res, 7)), Some((res == 0) as u8), Some(1));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn cp(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(rd);
    let rrv = cpu.reg(rr);
    let res = rdv.wrapping_sub(rrv);

    cpu.set_flags(None, None,
                   Some(bitneg(rdv, 3) & bit(rrv, 3)
                        | bit(rrv, 3) & bit(res, 3)
                        | bit(res, 3) & bitneg(rdv, 3)),
                   Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                        | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8),
                   Some(bitneg(rdv, 7) & bit(rrv, 7)
                        | bit(rrv, 7) & bit(res, 7)
                        | bit(res, 7) & bitneg(rdv, 7)));

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn cpc(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(rd);
    let rrv = cpu.reg(rr);
    let res = rdv.wrapping_sub(rrv).wrapping_sub(bit(cpu.flags(), C));

    let z = bit(cpu.flags(), Z);
    cpu.set_flags(None, None,
                   Some(bitneg(rdv, 3) & bit(rrv, 3)
                        | bit(rrv, 3) & bit(res, 3)
                        | bit(res, 3) & bitneg(rdv, 3)),
                   Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                        | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8 & z),
                   Some(bitneg(rdv, 7) & bit(rrv, 7)
                        | bit(rrv, 7) & bit(res, 7)
                        | bit(res, 7) & bitneg(rdv, 7)));

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn cpi(c: *mut Cpu, reg: Register, k: u8) {
    let cpu = unsafe {&mut *c};
    let rv = cpu.reg(reg);
    let res = rv.wrapping_sub(k);

    cpu.set_flags(None, None, Some(bitneg(rv, 3) & bit(k, 3)
                                    | bit(k, 3) & bit(res, 3) | bit(res, 3) & bitneg(rv, 3)),
                   Some(bit(rv, 7) & bitneg(k, 7) & bitneg(res, 7)
                        | bitneg(rv, 7) & bit(k, 7) & bit(res, 7)),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8),
                   Some(bitneg(rv, 7) & bit(k, 7)
                        | bit(k, 7) & bit(res, 7)
                        | bit(res, 7) & bitneg(rv, 7)));

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn cpse(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    if cpu.reg(rd) == cpu.reg(rr) {
        cpu.ip += 2;
        if cpu.mem.get_instruction(cpu.ip) == SecondOpWord {
            cpu.ip += 1;
        }
    } else {
        cpu.ip += 1;
    }
}
#[cfg(feature = "jit")]
extern "sysv64" fn c_sbi(c: *mut Cpu, typ: u8, ioreg: u8, b: u8) {
    let cpu = unsafe {&mut *c};
    let val = cpu.mem.io_reg(ioreg);
    cpu.mem.set_io_reg(ioreg, (val & !(1 << b)) | (typ << b));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn dec(c: *mut Cpu, reg: Register) {
    let cpu = unsafe {&mut *c};
    let res = cpu.reg(reg).wrapping_sub(1);
    *cpu.reg_mut(reg) = res;
    cpu.set_flags(None, None, None, Some((res == 0x7f) as u8),
                   Some(bit(res, 7)), Some((res == 0) as u8), None);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn eor(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let res = cpu.reg(rd) ^ cpu.reg(rr);
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(None, None, None, Some(0), Some(bit(res, 7)),
                   Some((res == 0) as u8), None);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn icall(c: *mut Cpu) {
    let cpu = unsafe {&mut *c};
    let retip = (cpu.ip + 1) as u16;
    cpu.mem.push16(retip);

    let ip = cpu.get_word_reg(data::Z);
    cpu.ip = ip as usize;
}
#[cfg(feature = "jit")]
extern "sysv64" fn in_(c: *mut Cpu, reg: Register, index: u8) {
    let cpu = unsafe {&mut *c};
    *cpu.reg_mut(reg) = cpu.mem.io_reg(index);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn jmp(c: *mut Cpu, ip: u32) {
    let cpu = unsafe {&mut *c};
    cpu.ip = ip as usize;
}
#[cfg(feature = "jit")]
extern "sysv64" fn ld_st(c: *mut Cpu, typ: u8, reg: Register, addrreg: Register, mode_u16: u16) {
    let cpu = unsafe {&mut *c};
    let mode = LDMode::from_u16(mode_u16);
    let displacement = match mode {
        LDMode::PreDecrement => {
            let modval = cpu.get_word_reg(addrreg).wrapping_sub(1);
            cpu.set_word_reg(addrreg, modval);
            0
        },
        LDMode::PostIncrement => 0,
        LDMode::Displacement(i) => i
    };

    let addr = cpu.get_word_reg(addrreg).wrapping_add(displacement as u16);
    match LDType::from_u8(typ) {
        LDType::LD => {
            *cpu.reg_mut(reg) = cpu.mem.data(addr);
        },
        LDType::ST => {
            let val = cpu.reg(reg);
            cpu.mem.set_data(addr, val);
        }
    }

    match mode {
        LDMode::PostIncrement => {
            let modval = cpu.get_word_reg(addrreg).wrapping_add(1);
            cpu.set_word_reg(addrreg, modval);
        },
        _ => {}
    };
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn lds(c: *mut Cpu, reg: Register, k: u16) {
    let cpu = unsafe {&mut *c};
    let val = cpu.mem.data(k);
    *cpu.reg_mut(reg) = val;
    cpu.ip += 2;
}
#[cfg(feature = "jit")]
extern "sysv64" fn sts(c: *mut Cpu, reg: Register, k: u16) {
    let cpu = unsafe {&mut *c};
    let val = cpu.reg(reg);
    cpu.mem.set_data(k, val);
    cpu.ip += 2;
}
#[cfg(feature = "jit")]
extern "sysv64" fn ldi(c: *mut Cpu, reg: Register, val: u8) {
    let cpu = unsafe {&mut *c};
    *cpu.reg_mut(reg) = val;
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn lpm(c: *mut Cpu, reg: Register, typ: u8) {
    let cpu = unsafe {&mut *c};
    let z = cpu.get_word_reg(data::Z);
    *cpu.reg_mut(reg) = cpu.mem.read_program(z);

    match LPMType::from_u8(typ) {
        LPMType::ZPostIncrement => cpu.set_word_reg(data::Z, z.wrapping_add(1)),
        _ => {}
    }

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn lsr(c: *mut Cpu, reg: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(reg);
    let res = rdv >> 1;
    *cpu.reg_mut(reg) = res;
    cpu.set_flags(None, None, None, Some(bit(rdv, 0)),
                   Some(0), Some((res == 0) as u8),
                   Some(bit(rdv, 0)));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn mov(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    *cpu.reg_mut(rd) = cpu.reg(rr);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn movw(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let val = cpu.get_word_reg(rr);
    cpu.set_word_reg(rd, val);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn mul(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let res = (cpu.reg(rd) as u16) * (cpu.reg(rr) as u16);
    cpu.set_word_reg(0, res);
    cpu.set_flags(None, None, None, None, None, Some((res == 0) as u8), Some(bit16(res, 15)));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn neg(c: *mut Cpu, rd: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(rd);
    let res = (0 as i8).wrapping_sub(rdv as i8) as u8;
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(None, None, Some(bit(res, 3) | bitneg(rdv, 3)),
                   Some((res == 0x80) as u8),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8),
                   Some((res != 0) as u8));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn nop(c: *mut Cpu) {
    let cpu = unsafe {&mut *c};
    if cpu.halt_on_nop {
        cpu.should_halt = true;
    }

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn or(c: *mut Cpu, rd: Register, rv: Register) {
    let cpu = unsafe {&mut *c};
    let res = cpu.reg(rd) | cpu.reg(rv);
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(None, None, None, Some(0), Some(bit(res, 7)), Some((res == 0) as u8), None);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn ori(c: *mut Cpu, rd: Register, k: u8) {
    let cpu = unsafe {&mut *c};
    let res = cpu.reg(rd) | k;
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(None, None, None, Some(0), Some(bit(res, 7)), Some((res == 0) as u8), None);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn out(c: *mut Cpu, reg: Register, index: u8) {
    let cpu = unsafe {&mut *c};
    let val = cpu.reg(reg);
    cpu.mem.set_io_reg(index, val);
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn pop(c: *mut Cpu, reg: Register) {
    let cpu = unsafe {&mut *c};
    let val = cpu.mem.pop();
    *cpu.reg_mut(reg) = val;

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn push(c: *mut Cpu, reg: Register) {
    let cpu = unsafe {&mut *c};
    let val = cpu.reg(reg);
    cpu.mem.push(val);

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn rcall(c: *mut Cpu, diff: i16) {
    let cpu = unsafe {&mut *c};
    let retip = (cpu.ip + 1) as u16;
    cpu.mem.push16(retip);

    cpu.ip = (cpu.ip as i32 + diff as i32) as usize;
}
#[cfg(feature = "jit")]
extern "sysv64" fn ret(c: *mut Cpu) {
    let cpu = unsafe {&mut *c};
    cpu.ip = cpu.mem.pop16() as usize;
}
#[cfg(feature = "jit")]
extern "sysv64" fn reti(c: *mut Cpu) {
    let cpu = unsafe {&mut *c};
    cpu.set_flags(Some(1), None, None, None, None, None, None);
    cpu.ip = cpu.mem.pop16() as usize;
}
#[cfg(feature = "jit")]
extern "sysv64" fn rjmp(c: *mut Cpu, diff: i16) {
    let cpu = unsafe {&mut *c};
    cpu.ip = (cpu.ip as i32 + diff as i32) as usize;
}
#[cfg(feature = "jit")]
extern "sysv64" fn ror(c: *mut Cpu, reg: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(reg);
    let res = bit(cpu.flags(), C) << 7 | (rdv >> 1);
    *cpu.reg_mut(reg) = res;
    cpu.set_flags(None, None, None, Some(bit(res, 7) ^ bit(rdv, 0)),
                   Some(bit(res, 7)), Some((res == 0) as u8),
                   Some(bit(rdv, 0)));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn sbci(c: *mut Cpu, reg: Register, val: u8) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(reg);
    let rrv = val;
    let res = rdv.wrapping_sub(rrv).wrapping_sub(bit(cpu.flags(), C));
    *cpu.reg_mut(reg) = res;

    let z = bit(cpu.flags(), Z);
    cpu.set_flags(None, None,
                   Some(bitneg(rdv, 3) & bit(rrv, 3)
                       | bit(rrv, 3) & bit(res, 3)
                       | bit(res, 3) & bitneg(rdv, 3)),
                   Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                       | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8 & z),
                   Some(bitneg(rdv, 7) & bit(rrv, 7)
                       | bit(rrv, 7) & bit(res, 7)
                       | bit(res, 7) & bitneg(rdv, 7)));

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn sbc(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(rd);
    let rrv = cpu.reg(rr);
    let res = rdv.wrapping_sub(rrv).wrapping_sub(bit(cpu.flags(), C));
    *cpu.reg_mut(rd) = res;

    let z = bit(cpu.flags(), Z);
    cpu.set_flags(None, None,
                   Some(bitneg(rdv, 3) & bit(rrv, 3)
                       | bit(rrv, 3) & bit(res, 3)
                       | bit(res, 3) & bitneg(rdv, 3)),
                   Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                       | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8 & z),
                   Some(bitneg(rdv, 7) & bit(rrv, 7)
                       | bit(rrv, 7) & bit(res, 7)
                       | bit(res, 7) & bitneg(rdv, 7)));

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn sbiw(c: *mut Cpu, reg: Register, k: u8) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.get_word_reg(reg);
    let res = rdv.wrapping_sub(k as u16);
    cpu.set_word_reg(reg, res);

    cpu.set_flags(None, None, None,
                   Some(bit16(rdv, 15) & bitneg16(res, 15)),
                   Some(bit16(res, 15)), Some((res == 0) as u8),
                   Some(bitneg16(rdv, 15) & bit16(res, 15)));
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn sbic_s(c: *mut Cpu, setclear: u8, reg: Register, b: u8) {
    let cpu = unsafe {&mut *c};
    if bit(cpu.mem.io_reg(reg), b as usize) == setclear {
        cpu.ip += 2;
        if cpu.mem.get_instruction(cpu.ip) == SecondOpWord {
            cpu.ip += 1;
        }
    } else {
        cpu.ip += 1;
    }
}
#[cfg(feature = "jit")]
extern "sysv64" fn sbr(c: *mut Cpu, setclear: u8, reg: Register, b: u8) {
    // TODO really doing the right thing or copy paste mistake
    let cpu = unsafe {&mut *c};
    if bit(cpu.reg(reg), b as usize) == setclear {
        cpu.ip += 2;
        if cpu.mem.get_instruction(cpu.ip) == SecondOpWord {
            cpu.ip += 1;
        }
    } else {
        cpu.ip += 1;
    }
}
#[cfg(feature = "jit")]
extern "sysv64" fn sleep(c: *mut Cpu) {
    let cpu = unsafe {&mut *c};
    cpu.sleeping = true;
    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn sub(c: *mut Cpu, rd: Register, rr: Register) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(rd);
    let rrv = cpu.reg(rr);
    let res = rdv.wrapping_sub(rrv);
    *cpu.reg_mut(rd) = res;
    cpu.set_flags(None, None,
                   Some(bitneg(rdv, 3) & bit(rrv, 3)
                       | bit(rrv, 3) & bit(res, 3)
                       | bit(res, 3) & bitneg(rdv, 3)),
                   Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                       | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8),
                   Some(bitneg(rdv, 7) & bit(rrv, 7)
                       | bit(rrv, 7) & bit(res, 7)
                       | bit(res, 7) & bitneg(rdv, 7)));

    cpu.ip += 1;
}
#[cfg(feature = "jit")]
extern "sysv64" fn subi(c: *mut Cpu, reg: Register, val: u8) {
    let cpu = unsafe {&mut *c};
    let rdv = cpu.reg(reg);
    let rrv = val;
    let res = rdv.wrapping_sub(rrv);
    *cpu.reg_mut(reg) = res;
    cpu.set_flags(None, None,
                   Some(bitneg(rdv, 3) & bit(rrv, 3)
                       | bit(rrv, 3) & bit(res, 3)
                       | bit(res, 3) & bitneg(rdv, 3)),
                   Some(bit(rdv, 7) & bitneg(rrv, 7) & bitneg(res, 7)
                       | bitneg(rdv, 7) & bit(rrv, 7) & bit(res, 7)),
                   Some(bit(res, 7)),
                   Some((res == 0) as u8),
                   Some(bitneg(rdv, 7) & bit(rrv, 7)
                       | bit(rrv, 7) & bit(res, 7)
                       | bit(res, 7) & bitneg(rdv, 7)));

    cpu.ip += 1;
}


#[cfg(test)]
mod tests {
    use memory::Memory;
    use util::{assemble_to_file};
    use super::{Cpu};

    macro_rules! check {
        ($code: expr; reg: $($reg: expr => $regval: expr), *; expect: $($regexp: expr => $regexpval: expr), *; flags: $flagval: expr) => {{
            let mut cpu = create($code);
            $(
                *cpu.reg_mut($reg) = $regval;
            )*
                while cpu.step() {}
            $(
                assert_eq!(cpu.reg($regexp), $regexpval);
            )*
                assert_eq!(cpu.flags(), $flagval, "flags: {:#b}", cpu.flags());
        }}
    }

    #[test]
    fn adc() {
        check!("add r0, r2\nadc r1, r3";
               reg: 0 => 255, 1 => 255, 2 => 255, 3 => 254;
               expect: 0 => 254, 1 => 254;
               flags: 0b00110101);
    }

    #[test]
    fn add() {
        check!("add r0, r1";
               reg: 0 => 10, 1 => 7;
               expect: 0 => 17, 1 => 7;
               flags: 0b00100000);
        check!("add r0, r1";
               reg: 0 => 128, 1 => 128;
               expect: 0 => 0, 1 => 128;
               flags: 0b00011011);
        check!("add r0, r1";
               reg: 0 => 255, 1 => 255;
               expect: 0 => 254, 1 => 255;
               flags: 0b00110101);
        check!("add r0, r1";
               reg: 0 => 127, 1 => 127;
               expect: 0 => 254, 1 => 127;
               flags: 0b00101100);
    }

    #[test]
    fn adiw() {
        check!("adiw r24, 63";
               reg: 24 => 247, 25 => 255;
               expect: 24 => 54, 25 => 0;
               flags: 0b00000001);

        check!("adiw r24, 63";
               reg: 24 => 255, 25 => 127;
               expect: 24 => 62, 25 => 128;
               flags: 0b00001100);
    }

    #[test]
    fn and() {
        check!("and r0, r1";
               reg: 0 => 0b10101010, 1 => 0b10001111;
               expect: 0 => 0b10001010, 1 => 0b10001111;
               flags: 0b00010100);
        check!("and r0, r1";
               reg: 0 => 0b11110000, 1 => 0b00001111;
               expect: 0 => 0, 1 => 0b00001111;
               flags: 0b00000010);
    }

    #[test]
    fn andi() {
        check!("andi r16, 143";
               reg: 16 => 0b10101010;
               expect: 16 => 0b10001010;
               flags: 0b00010100);
        check!("andi r16, 15";
               reg: 16 => 0b11110000;
               expect: 16 => 0;
               flags: 0b00000010);
    }

    #[test]
    fn asr() {
        check!("asr r0";
               reg: 0 => (-2 as i8) as u8;
               expect: 0 => (-1 as i8) as u8;
               flags: 0b00001100);
        check!("asr r0";
               reg: 0 => 0b10000101;
               expect: 0 => 0b11000010;
               flags: 0b00010101);
    }

    #[test]
    fn bclr() {
        check!("out 0x3f, r0\nbclr 4";
               reg: 0 => 0xff;
               expect: 0 => 0xff;
               flags: 0b11101111);

        check!("bset 3\n\nbclr 4";
               reg: 0 => 1;
               expect: 0 => 1;
               flags: 0b00001000);
    }

    #[test]
    fn bld() {
        check!("bst 0, 0\nbld 1, 1";
               reg: 0 => 1, 1 => 1;
               expect: 0 => 1, 1 => 3;
               flags: 0b01000000);
    }

    #[test]
    fn bst() {
        check!("bst 0, 0";
               reg: 0 => 1;
               expect: 0 => 1;
               flags: 0b01000000);
    }


    #[test]
    fn bset() {
        check!("bset 3";
               reg: 0 => 1;
               expect: 0 => 1;
               flags: 0b00001000);
    }

    #[test]
    fn brbc() {
        check!("add r0, r0\nbrbc 0, d\nadd r0, r1\nd:add r0, r2";
               reg: 0 => 0, 1 => 1, 2 => 2;
               expect: 0 => 2;
               flags: 0);
    }

    #[test]
    fn brbs() {
        check!("add r5, r5\nbrbs 0, d\nadd r0, r1\nd:add r0, r2";
               reg: 0 => 0, 1 => 1, 2 => 2, 5 => 255;
               expect: 0 => 2;
               flags: 0);
    }

    #[test]
    fn call() {
        check!("out 0x3e, r5\nout 0x3d, r6\ncall d\nadd r0, r1\nd:add r0, r2";
               reg: 0 => 0, 1 => 1, 2 => 2, 5 => 0x8, 6 => 0x0;
               expect: 0 => 2;
               flags: 0);
    }

    #[test]
    fn cbi() {
        check!("out 30, r0\ncbi 30, 2\nin r1, 30";
               reg: 0 => 0xff;
               expect: 0 => 0xff, 1 => 0b11111011;
               flags: 0);
    }


    #[test]
    fn com() {
        check!("com r0";
               reg: 0 => 0;
               expect: 0 => 0xff;
               flags: 0b00010101);
        check!("com r0";
               reg: 0 => 0xff;
               expect: 0 => 0x0;
               flags: 0b00000011);
        check!("com r0";
               reg: 0 => 0b11001010;
               expect: 0 => 0b00110101;
               flags: 0b00000001);
    }

    #[test]
    fn cp() {
        check!("cp r16, r17";
               reg: 16 => 0, 17 => 127;
               expect: 16 => 0;
               flags: 0b00110101);

        check!("cp r16, r17";
               reg: 16 => 255, 17 => 127;
               expect: 16 => 255;
               flags: 0b00010100);

        check!("cp r16, r17";
               reg: 16 => 1, 17 => 128;
               expect: 16 => 1;
               flags: 0b00001101);
    }

    #[test]
    fn cpc() {
        check!("cpi R28, 128\ncpc R29, R28";
               reg: 28 => 0, 29 => 0;
               expect: 28 => 0, 29 => 0;
               flags: 0b00110101);
    }

    #[test]
    fn cpi() {
        check!("cpi R28, 128";
               reg: 28 => 0;
               expect: 28 => 0;
               flags: 0b00001101);
    }

    #[test]
    fn cpse() {
        check!("cpse r0, r1\nadd r2, r3\nadd r2, r3";
               reg: 0 => 0, 1 => 1, 2 => 0, 3 => 1;
               expect: 2 => 2;
               flags: 0);

        check!("cpse r0, r1\nadd r2, r3\nadd r2, r3";
               reg: 0 => 0, 1 => 0, 2 => 0, 3 => 1;
               expect: 2 => 1;
               flags: 0);

        check!("d:cpse r0, r1\ncall d\nadd r2, r3";
               reg: 0 => 1, 1 => 1, 2 => 0, 3 => 1;
               expect: 2 => 1;
               flags: 0);
    }

    #[test]
    fn dec() {
        check!("dec r0";
               reg: 0 => 0;
               expect: 0 => 255;
               flags: 0b00010100);

        check!("dec r0";
               reg: 0 => 0x80;
               expect: 0 => 0x7f;
               flags: 0b00011000);

        check!("dec r0";
               reg: 0 => 1;
               expect: 0 => 0;
               flags: 0b00000010);
    }

    #[test]
    fn eor() {
        check!("eor r28, r29";
               reg: 28 => 123, 29 => 123;
               expect: 28 => 0, 29 => 123;
               flags: 0b00000010);
        check!("eor r28, r29";
               reg: 28 => 0b10101010, 29 => 0b1111;
               expect: 28 => 0b10100101, 29 => 0b1111;
               flags: 0b00010100);
    }

    #[test]
    fn icall() {
        check!("out 0x3e, r5\nout 0x3d, r6\nicall\nadd r0, r1\nd:add r0, r2";
               reg: 0 => 0, 1 => 1, 2 => 2, 5 => 0x8, 6 => 0x0, 30 => 4, 31 => 0;
               expect: 0 => 2;
               flags: 0);
    }


    #[test]
    fn in_() {
        check!("eor r1, r1\nin r0, 0x3f";
               reg: 0 => 0xff;
               expect: 0 => 0b10;
               flags: 0b10);

        check!("out 0x10, r1\nin r0, 0x10";
               reg: 0 => 0, 1 => 1;
               expect: 0 => 1;
               flags: 0);
    }

    #[test]
    fn jmp() {
        check!("add r0, r0\njmp d\nadd r0, r1\nd:add r0, r2";
               reg: 0 => 0, 1 => 1, 2 => 2;
               expect: 0 => 2;
               flags: 0);
    }

    #[test]
    fn ld() {
        check!("ld r0, Y+";
               reg: 0 => 0, 1 => 1, 28 => 0x1, 29 => 0;
               expect: 0 => 1, 1 => 1, 28 => 0x2, 29 => 0;
               flags: 0);
        check!("ld r0, -X";
               reg: 0 => 0, 8 => 1, 26 => 0x9, 27 => 0;
               expect: 0 => 1, 8 => 1, 26 => 0x8, 27 => 0;
               flags: 0);

        check!("eor r1, r1\nldd r0, Z+63";
               reg: 30 => 0x20, 31 => 0;
               expect: 0 => 0x2, 30 => 0x20, 31 => 0;
               flags: 0b10);
    }

    #[test]
    fn ldi() {
        check!("ldi r28, 128";
               reg: 28 => 1;
               expect: 28 => 128;
               flags: 0);
    }

    #[test]
    fn lds() {
        check!("lds r0, 1";
               reg: 0 => 0, 1 => 1;
               expect: 0 => 1;
               flags: 0);
    }


    #[test]
    fn lpm() {
        check!("lpm r28, Z+";
               reg: 30 => 1, 31 => 0;
               expect: 30 => 2, 31 => 0, 28 => 145;
               flags: 0);
    }

    #[test]
    fn lsl() {
        check!("lsl r0";
               reg: 0 => 0b11001011;
               expect: 0 => 0b10010110;
               flags: 0b00110101);
    }

    #[test]
    fn lsr() {
        check!("lsr r0";
               reg: 0 => 0b101;
               expect: 0 => 0b10;
               flags: 0b00011001);
    }

    #[test]
    fn mov() {
        check!("mov r0, r1";
               reg: 1 => 0x12;
               expect: 0 => 0x12, 1 => 0x12;
               flags: 0);
    }

    #[test]
    fn movw() {
        check!("movw r0, r2";
               reg: 2 => 0x12, 3 => 0x34;
               expect: 0 => 0x12, 1 => 0x34, 2 => 0x12, 3 => 0x34;
               flags: 0);
    }

    #[test]
    fn mul() {
        check!("mul r1, r2";
               reg: 1 => 0xff, 2 => 0xff;
               expect: 0 => 0x01, 1 => 0xfe;
               flags: 0b00000001);

        check!("mul r1, r2";
               reg: 1 => 3, 2 => 0;
               expect: 0 => 0, 1 => 0;
               flags: 0b00000010);

        check!("mul r1, r2";
               reg: 1 => 123, 2 => 4;
               expect: 0 => 236, 1 => 1;
               flags: 0b00000000);

    }

    #[test]
    fn neg() {
        check!("neg r0";
               reg: 0 => 0;
               expect: 0 => 0;
               flags: 0b00100010);

        check!("neg r0";
               reg: 0 => 255;
               expect: 0 => 1;
               flags: 0b00000001);

        check!("neg r0";
               reg: 0 => 0x80;
               expect: 0 => 0x80;
               flags: 0b00101101);

        check!("neg r0";
               reg: 0 => 1;
               expect: 0 => 255;
               flags: 0b00110101);
    }

    #[test]
    fn out() {
        check!("out 0x3f, r0";
               reg: 0 => 0xff;
               expect: 0 => 0xff;
               flags: 0xff);
    }

    #[test]
    fn or() {
        check!("or r0, r1";
               reg: 0 => 0b10101010, 1 => 0b10001111;
               expect: 0 => 0b10101111, 1 => 0b10001111;
               flags: 0b00010100);
        check!("or r0, r1";
               reg: 0 => 0, 1 => 0;
               expect: 0 => 0, 1 => 0;
               flags: 0b00000010);
    }

    #[test]
    fn ori() {
        check!("ori r16, 143";
               reg: 16 => 0b10101010;
               expect: 16 => 0b10101111;
               flags: 0b00010100);
        check!("ori r16, 0";
               reg: 16 => 0;
               expect: 16 => 0;
               flags: 0b00000010);
    }

    #[test]
    fn pop() {
        check!("out 0x3e, r5\nout 0x3d, r6\nst X, r1\npop r0";
               reg: 0 => 0, 1 => 1, 5 => 0x8, 6 => 0x0, 26 => 1, 27 => 8;
               expect: 0 => 1;
               flags: 0);
    }

    #[test]
    fn push() {
        check!("out 0x3e, r5\nout 0x3d, r6\npush r1\npop r0";
               reg: 0 => 0, 1 => 1, 5 => 0x8, 6 => 0x0;
               expect: 0 => 1;
               flags: 0);
    }

    #[test]
    fn rcall() {
        check!("out 0x3e, r5\nout 0x3d, r6\nrcall d\nadd r0, r1\nd:add r0, r2";
               reg: 0 => 0, 1 => 1, 2 => 2, 5 => 0x8, 6 => 0x0;
               expect: 0 => 2;
               flags: 0);
    }

    #[test]
    fn ret() {
        check!("out 0x3e, r5\nout 0x3d, r6\ncall d\nmov r0, r1\nnop\nd:ret";
               reg: 0 => 0, 1 => 1, 5 => 0x8, 6 => 0x0;
               expect: 0 => 1;
               flags: 0);

        check!("out 0x3e, r5\nout 0x3d, r6\nrcall d\nmov r0, r1\nnop\nd:ret";
               reg: 0 => 0, 1 => 1, 5 => 0x8, 6 => 0x0;
               expect: 0 => 1;
               flags: 0);

        check!("out 0x3e, r5\nout 0x3d, r6\nicall\nmov r0, r1\nnop\nd:ret";
               reg: 0 => 0, 1 => 1, 5 => 0x8, 6 => 0x0, 30 => 5, 31 => 0;
               expect: 0 => 1;
               flags: 0);
    }

    #[test]
    fn reti() {
        check!("out 0x3e, r5\nout 0x3d, r6\ncall d\nmov r0, r1\nnop\nd:reti";
               reg: 0 => 0, 1 => 1, 5 => 0x8, 6 => 0x0;
               expect: 0 => 1;
               flags: 0b10000000);

        check!("out 0x3e, r5\nout 0x3d, r6\nrcall d\nmov r0, r1\nnop\nd:reti";
               reg: 0 => 0, 1 => 1, 5 => 0x8, 6 => 0x0;
               expect: 0 => 1;
               flags: 0b10000000);
    }

    #[test]
    fn rjmp() {
        check!("add r0, r0\nrjmp d\nadd r0, r1\nd:add r0, r2";
               reg: 0 => 0, 1 => 1, 2 => 2;
               expect: 0 => 2;
               flags: 0);
    }

    #[test]
    fn ror() {
        check!("out 0x3f, r1\n ror r0";
               reg: 0 => 0b01010001, 1 => 1;
               expect: 0 => 0b10101000;
               flags: 0b00010101);
    }

    #[test]
    fn rol() {
        check!("out 0x3f, r1\n rol r0";
               reg: 0 => 0b10011001, 1 => 1;
               expect: 0 => 0b00110011;
               flags: 0b00111001);
    }


    #[test]
    fn st() {
        check!("st Y+, r0";
               reg: 0 => 1, 28 => 0x1, 29 => 0;
               expect: 0 => 1, 1 => 1, 28 => 0x2, 29 => 0;
               flags: 0);
        check!("st -X, r0";
               reg: 0 => 1, 26 => 0x9, 27 => 0;
               expect: 0 => 1, 8 => 1, 26 => 0x8, 27 => 0;
               flags: 0);

        check!("std Z+63, r0";
               reg: 0 => 0xff, 30 => 0x20, 31 => 0;
               expect: 0 => 0xff, 30 => 0x20, 31 => 0;
               flags: 0xff);
    }

    #[test]
    fn sts() {
        check!("sts 1, r0";
               reg: 0 => 1, 1 => 0;
               expect: 1 => 1;
               flags: 0);
    }

    #[test]
    fn sbc() {
        check!("subi r16, 255\nsbc r17, r18";
               reg: 16 => 255, 17 => 255, 18 => 255;
               expect: 16 => 0, 17 => 0;
               flags: 0b00000010);

        check!("subi r16, 255\nsbc r17, r18";
               reg: 16 => 0, 17 => 255, 18 => 254;
               expect: 16 => 1, 17 => 0;
               flags: 0b00000000);

        check!("subi r16, 255\nsbc r17, r18";
               reg: 16 => 0, 17 => 0, 18 => 255;
               expect: 16 => 1, 17 => 0;
               flags: 0b00100001);
    }

    #[test]
    fn sbci() {
        check!("subi r16, 255\nsbci r17, 255";
               reg: 16 => 255, 17 => 255;
               expect: 16 => 0, 17 => 0;
               flags: 0b00000010);

        check!("subi r16, 255\nsbci r17, 254";
               reg: 16 => 0, 17 => 255;
               expect: 16 => 1, 17 => 0;
               flags: 0b00000000);

        check!("subi r16, 255\nsbci r17, 255";
               reg: 16 => 0, 17 => 0;
               expect: 16 => 1, 17 => 0;
               flags: 0b00100001);
    }

    #[test]
    fn sbi() {
        check!("sbi 30, 2\nin r0, 30";
               reg: 0 => 0;
               expect: 0 => 0b00000100;
               flags: 0);
    }

    #[test]
    fn sbic() {
        check!("out 30, r0\nsbic 30, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 1, 1 => 0, 2 => 1;
               expect: 1 => 1;
               flags: 0);

        check!("out 30, r0\nd:sbic 30, 1\ncall d\nadd r1, r2";
               reg: 0 => 0, 1 => 0, 2 => 1;
               expect: 1 => 1;
               flags: 0);

        check!("out 30, r0\nsbic 30, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 2, 1 => 0, 2 => 1;
               expect: 1 => 2;
               flags: 0);
    }

    #[test]
    fn sbis() {
        check!("out 30, r0\nsbis 30, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 1, 1 => 0, 2 => 1;
               expect: 1 => 2;
               flags: 0);

        check!("out 30, r0\nd:sbis 30, 1\ncall d\nadd r1, r2";
               reg: 0 => 2, 1 => 0, 2 => 1;
               expect: 1 => 1;
               flags: 0);

        check!("out 30, r0\nsbis 30, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 2, 1 => 0, 2 => 1;
               expect: 1 => 1;
               flags: 0);
    }

    #[test]
    fn sbiw() {
        check!("sbiw r24, 1";
               reg: 24 => 0;
               expect: 24 => 255;
               flags: 0b00010101);
    }

    #[test]
    fn sbrc() {
        check!("sbrc r0, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 0, 1 => 0, 2 => 1;
               expect: 1 => 1;
               flags: 0);

        check!("d:sbrc r0, 1\ncall d\nadd r1, r2";
               reg: 0 => 0, 1 => 0, 2 => 1;
               expect: 1 => 1;
               flags: 0);

        check!("sbrc r0, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 2, 1 => 0, 2 => 1;
               expect: 1 => 2;
               flags: 0);
    }

    #[test]
    fn sbrs() {
        check!("sbrs r0, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 0, 1 => 0, 2 => 1;
               expect: 1 => 2;
               flags: 0);

        check!("sbrs r0, 1\nadd r1, r2\nadd r1, r2";
               reg: 0 => 2, 1 => 0, 2 => 1;
               expect: 1 => 1;
               flags: 0);
    }

    #[test]
    fn sub() {
        check!("sub r16, r17";
               reg: 16 => 0, 17 => 127;
               expect: 16 => 129;
               flags: 0b00110101);

        check!("sub r16, r17";
               reg: 16 => 255, 17 => 127;
               expect: 16 => 128;
               flags: 0b00010100);

        check!("sub r16, r17";
               reg: 16 => 1, 17 => 128;
               expect: 16 => 129;
               flags: 0b00001101);
    }

    #[test]
    fn subi() {
        check!("subi r16, 127";
               reg: 16 => 0;
               expect: 16 => 129;
               flags: 0b00110101);

        check!("subi r16, 127";
               reg: 16 => 255;
               expect: 16 => 128;
               flags: 0b00010100);

        check!("subi r16, 128";
               reg: 16 => 1;
               expect: 16 => 129;
               flags: 0b00001101);
    }

    fn create(code: &str) -> Cpu {
        Cpu::new(Memory::new(assemble_to_file(code), None), true)
    }
}
