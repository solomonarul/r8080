mod buses;

use std::{fs::File, io::Read, sync::{Arc, RwLock}};

use buses::TestCPMBus;
use r8080::{cpu::{Interpreter8080, CPU8080}, Bus8080};

fn read_file_to_vec(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).unwrap();
    buffer
}

#[test]
fn test_tst8080_com()
{
    let mut bus = Box::new(TestCPMBus::new("MICROCOSM ASSOCIATES 8080/8085 CPU DIAGNOSTIC\x0D\x0A VERSION 1.0  (C) 1980\x0D\x0A\x0D\x0A CPU IS OPERATIONAL"));
    bus.write_buffer(0x0100, read_file_to_vec("test_roms/TST8080.COM"));

    let mut cpu = Box::new(Interpreter8080::new()) as Box<dyn CPU8080>;
    cpu.force_jump(0x100);
    cpu.set_bus(Arc::new(RwLock::new(bus)));
    cpu.run();
}

#[test]
fn test_cputest_com()
{
    let mut bus = Box::new(TestCPMBus::new("idk"));
    bus.write_buffer(0x0100, read_file_to_vec("test_roms/CPUTEST.COM"));

    let mut cpu = Box::new(Interpreter8080::new()) as Box<dyn CPU8080>;
    cpu.force_jump(0x100);
    cpu.set_bus(Arc::new(RwLock::new(bus)));
    cpu.run();
}

#[test]
fn test_8080pre_com()
{
    let mut bus = Box::new(TestCPMBus::new("8080 Preliminary tests complete"));
    bus.write_buffer(0x0100, read_file_to_vec("test_roms/8080PRE.COM"));

    let mut cpu: Box<dyn CPU8080> = Box::new(Interpreter8080::new());
    cpu.force_jump(0x100);
    cpu.set_bus(Arc::new(RwLock::new(bus)));
    cpu.run();
}

#[test]
fn test_8080exer_com()
{
    let mut bus = Box::new(TestCPMBus::new("idk"));
    bus.write_buffer(0x0100, read_file_to_vec("test_roms/8080EXER.COM"));

    let mut cpu: Box<dyn CPU8080> = Box::new(Interpreter8080::new());
    cpu.force_jump(0x100);
    cpu.set_bus(Arc::new(RwLock::new(bus)));
    cpu.run();
}