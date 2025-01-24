use std::any::Any;

use cpu::Registers;

pub mod cpu;

pub trait Bus8080: Any
{
    fn read_b(&self, a: u16) -> u8;
    fn read_w(&self, a: u16) -> u16;
    fn has_interrupt(&self) -> bool;
    fn get_interrupt(&mut self) -> u8;
    fn push_interrupt(&mut self, b: u8);
    fn write_b(&mut self, a: u16, b: u8);
    fn write_w(&mut self, a: u16, w: u16);
    fn in_b(&mut self, regs: &mut Registers, b: u8) -> u8;
    fn out_b(&mut self, regs: &mut Registers, b: u8, a: u8);
    fn write_buffer(&mut self, a: u16, data: Vec<u8>);
}

struct ErrorBus;
impl ErrorBus
{
    fn new() -> Self {
        Self {}
    }
}

impl Bus8080 for ErrorBus
{
    fn in_b(&mut self, _: &mut Registers, _: u8) -> u8 {
        panic!("Unimplemented Bus.");
    }

    fn get_interrupt(&mut self) -> u8 {
        panic!("Unimplemented Bus.");
    }

    fn has_interrupt(&self) -> bool {
        panic!("Unimplemented Bus.");
    }

    fn push_interrupt(&mut self, _: u8) {
        panic!("Unimplemented Bus.");
    }

    fn out_b(&mut self, _: &mut Registers, _: u8, _: u8) {
        panic!("Unimplemented Bus.");
    }

    fn read_b(&self, _: u16) -> u8 {
        panic!("Unimplemented Bus.");
    }

    fn read_w(&self, _: u16) -> u16 {
        panic!("Unimplemented Bus.");
    }

    fn write_b(&mut self, _: u16, _: u8) {
        panic!("Unimplemented Bus.");
    }

    fn write_w(&mut self, _: u16, _: u16) {
        panic!("Unimplemented Bus.");
    }

    fn write_buffer(&mut self, _: u16, _: Vec<u8>) {
        panic!("Unimplemented Bus.");
    }
}