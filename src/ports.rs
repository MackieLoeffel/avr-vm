use io::{IO, HIGH, LOW};
use util::{bit, bits};

const PORT_OFFSET: u16 = 0x30;
const ADEN: usize = 7;
const ADSC: usize = 6;
const ADATE: usize = 5;
const ADIF: usize = 4;
const ADIE: usize = 3;
const REFS1: usize = 7;
const REFS0: usize = 6;
const ADLAR: usize = 5;
const MUX4: usize = 4;
const MUX3: usize = 3;
const ADC_BITS: usize = 10;
const ADMUX: usize = 0x27;
const ADCSRA: usize = 0x26;
const ADCH: usize = 0x25;
const ADCL: usize = 0x24;
const PORTA: usize = 0;
const ADC_PORT: usize = PORTA;
pub const PIND: usize = 0x30;

pub struct Port<'a> {
    io: Option<&'a IO>,
    ddr: u8,
    port: u8,
    index: u16,
}

#[derive(Debug)]
enum PortReg {
    PIN, PORT, DDR
}

impl<'a> Port<'a> {
    pub fn new(io: Option<&'a IO>, index: u16) -> Port<'a> {
        Port {io: io, index: index, ddr: 0, port: 0}
    }

    #[inline]
    pub fn read(&self, index: u16) -> Option<u8> {
        let typ = try_opt!(self.typ(index));
        let io = try_opt!(self.io);
        match typ {
            PortReg::DDR => Some(self.ddr),
            PortReg::PIN => {
                let mut ret = 0;
                for i in (0..8).rev() {
                    let b = if self.is_output(i) {
                        // reading from pin if set to output is undefined?
                        0
                    } else {
                        io.p[self.index as usize][i].as_bin()
                    };

                    ret = ret << 1 | b;
                }
                Some(ret)
            }
            _ => None,
        }
    }

    #[inline]
    pub fn write(&mut self, index: u16, val: u8) {
        let typ = try_opt_void!(self.typ(index));
        let io = try_opt_void!(self.io);
        if let PortReg::PIN = typ {
            return;
        }

        match typ {
            PortReg::DDR => { self.ddr = val; },
            PortReg::PORT => { self.port = val; },
            _ => unreachable!()
        }

        for i in (0..8).rev() {
            if !self.is_output(i) {
                continue;
            }

            let output = if bit(self.port, i) == 1 { HIGH } else { LOW };
            io.p[self.index as usize][i].set(output);
        }
    }

    #[inline]
    fn typ(&self, index: u16) -> Option<PortReg> {
        if index < PORT_OFFSET + (3 - self.index) * 3
            || index >= PORT_OFFSET + (3 - self.index + 1) * 3 {
                return None;
            }
        // works because PORT_OFFSET % 3 == 0
        Some(match index % 3 {
            0 => PortReg::PIN,
            1 => PortReg::DDR,
            2 => PortReg::PORT,
            _ => unreachable!()
        })
    }

    #[inline]
    fn is_output(&self, num_pin: usize) -> bool {
        bit(self.ddr, num_pin) == 1
    }
}

pub fn adc_write(io: &IO, data: &mut [u8], index: u16, val: u8) {
    if index as usize != ADCSRA {
        return;
    }

    if bit(val, ADEN) == 1 && bit(val, ADSC) == 1 {
        // we use vcc as avcc, because we don't have a special wire for it
        // we only allow avcc as reference
        // we also ignore the prescaler
        // ADLAR must be set to right adjust
        // MUX3, MUX4, ADATE, ADIF and ADIE must be cleared
        let admux = data[ADMUX];
        assert!(bit(admux, REFS1) == 0 && bit(admux, REFS0) == 1);
        assert!(bit(admux, ADLAR) == 0);
        assert!(bit(admux, MUX3) == 0 && bit(admux, MUX4) == 0);
        assert!(bit(val, ADATE) == 0 && bit(val, ADIF) == 0 && bit(val, ADIE) == 0);

        let pin = bits(admux as u16, 0, 3);
        let mut read = (io.p[ADC_PORT][pin as usize].mv() as u32)
            * (1 << ADC_BITS) / (io.vcc.mv() as u32);
        if read == (1 << ADC_BITS) {
            read -= 1;
        }
        println!("Read ADC {}", read);

        data[ADCL] = read as u8;
        data[ADCH] = bits(read as u16, 8, 2) as u8;
        data[index as usize] = val & !(1 << ADSC);
    }
}
