use memory::{Memory};
use ports::{PIND};
use util::{bit, bits};

const GIFR: u16 = 0x5A;
const INTF0: usize = 6;
const INTF1: usize = 7;
const INT_PIN_BASE: usize = 2;
const MCUCR: u16 = 0x55;
const GICR: u16 = 0x5B;
const ISC_BASE: usize = 0;
const INT_BASE: usize = 6;

const TIFR: u16 = 0x58;
const TIMSK: u16 = 0x59;
const OCIE1A: usize = 4;
const OCF1A: usize = 4;
const OCR1A: u8 = 0x2a;
const TCNT1: u8 = 0x2c;
const TCCR1B: u16 = 0x4e;
const CS1: u8 = 0;

pub struct PortInterrupts {
    // saves the previous value of the ports
    // used for edge detection
    prev: [u8; 2]
}

impl PortInterrupts {
    pub fn new() -> PortInterrupts {
        PortInterrupts {prev: [0; 2]}
    }

    #[inline(always)]
    pub fn step(&mut self, mem: &mut Memory) {
        for int_nr in 0..2 {
            if bit(mem.data(GICR), INT_BASE + int_nr) == 0 { continue; }

            let sense_ctrl = bits(mem.data(MCUCR) as u16, (ISC_BASE + int_nr * 2) as u8, 2);
            let new_val = bit(mem.data(PIND as u16), INT_PIN_BASE + int_nr);
            let triggered = match sense_ctrl {
                0 => { new_val == 0 }
                1 => { new_val != self.prev[int_nr] }
                _ => panic!("not implemented sense ctrl: {}", sense_ctrl)
            };
            self.prev[int_nr] = new_val;
            if triggered {
                let gifr = mem.data(GIFR);
                mem.set_data(GIFR, gifr | (1 << (INT_BASE + int_nr)));
            }
        }
    }

    #[inline(always)]
    pub fn pending_interrupt(&mut self, mem: &mut Memory) -> Option<usize> {
        let gifr = mem.data(GIFR);
        if bit(gifr, INTF0) == 1 {
            // we can be sure, that the interrupt gets handled,
            // when we return Some
            mem.set_data(GIFR, gifr & !(1 << INTF0));
            return Some(1);
        }
        if bit(gifr, INTF1) == 1 {
            mem.set_data(GIFR, gifr & !(1 << INTF1));
            return Some(2);
        }
        None
    }
}

pub struct TimerInterrupts {
    steps: u32
}

impl TimerInterrupts {
    pub fn new() -> TimerInterrupts {
        TimerInterrupts {steps: 0}
    }

    #[inline(always)]
    pub fn step(&mut self, mem: &mut Memory) {
        let clock_select = bits(mem.data(TCCR1B) as u16, CS1, 3);
        if clock_select == 0 { return; }
        self.steps += 1;
        let prescaler = match clock_select {
            0b001 => 1,
            0b010 => 8,
            0b011 => 64,
            0b100 => 256,
            0b101 => 1024,
            _ => panic!("Unsupported clock select: {}", clock_select)
        };
        if self.steps >= prescaler { // tick real timer?
            self.steps = 0;
            // only ctc mode
            let mut timer_val = mem.io_reg16(TCNT1);
            timer_val += 1;
            if timer_val == mem.io_reg16(OCR1A) {
                timer_val = 0;

                let tifr = mem.data(TIFR);
                mem.set_data(TIFR, tifr | (1 << OCF1A));
            }
            mem.set_io_reg16(TCNT1, timer_val);
        }
    }

    #[inline(always)]
    pub fn pending_interrupt(&mut self, mem: &mut Memory) -> Option<usize> {
        let tifr = mem.data(TIFR);
        if bit(mem.data(TIMSK), OCIE1A) == 1 && bit(tifr, OCF1A) == 1 {
            mem.set_data(TIFR, tifr & !(1 << OCF1A));
            return Some(7);
        }
        None
    }
}
