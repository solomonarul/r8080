#![feature(trait_upcasting)]
mod buses;

use std::{any::Any, fs::File, io::Read};

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
    let mut tst8080_bus = Box::new(TestCPMBus::new());
    tst8080_bus.write_buffer(0x0100, read_file_to_vec("test_roms/TST8080.COM"));

    // Write some patches for the ROM to actually do something in coordination with the TestBus that is already implemented.
    tst8080_bus.write_buffer(0x0000, [0xD3, 0x00].to_vec());                                // Stop.
    tst8080_bus.write_buffer(0x0005, [0xD3, 0x01, 0xC9].to_vec());                          // Print and ret.

    let mut cpu: Box<dyn CPU8080> = Box::new(Interpreter8080::new());
    cpu.force_jump(0x100);
    cpu.set_bus(tst8080_bus);
    cpu.run();

    let bus: Box<TestCPMBus> = (cpu.remove_bus() as Box<dyn Any>).downcast::<TestCPMBus>().unwrap();
    if bus.get_output() != "MICROCOSM ASSOCIATES 8080/8085 CPU DIAGNOSTIC\x0D\x0A VERSION 1.0  (C) 1980\x0D\x0A\x0D\x0A CPU IS OPERATIONAL".as_bytes()
    {
        for char in bus.get_output()
        {
            print!("{}", *char as char);
        }
        panic!("[EROR]: Test ended with wrong output.");
    }
}

#[test]
fn test_cputest_com()
{
    let mut tst8080_bus = Box::new(TestCPMBus::new());
    tst8080_bus.write_buffer(0x0100, read_file_to_vec("test_roms/CPUTEST.COM"));

    // Write some patches for the ROM to actually do something in coordination with the TestBus that is already implemented.
    tst8080_bus.write_buffer(0x0000, [0xD3, 0x00].to_vec());                                // Stop.
    tst8080_bus.write_buffer(0x0005, [0xD3, 0x01, 0xC9].to_vec());                          // Print and ret.

    let mut cpu: Box<dyn CPU8080> = Box::new(Interpreter8080::new());
    cpu.force_jump(0x100);
    cpu.set_bus(tst8080_bus);
    cpu.run();

    let bus: Box<TestCPMBus> = (cpu.remove_bus() as Box<dyn Any>).downcast::<TestCPMBus>().unwrap();
    if bus.get_output() != "idk".as_bytes()
    {
        for char in bus.get_output()
        {
            print!("{}", *char as char);
        }
        panic!("[EROR]: Test ended with wrong output.");
    }
}

#[test]
fn test_8080pre_com()
{
    let mut tst8080_bus = Box::new(TestCPMBus::new());
    tst8080_bus.write_buffer(0x0100, read_file_to_vec("test_roms/8080PRE.COM"));

    // Write some patches for the ROM to actually do something in coordination with the TestBus that is already implemented.
    tst8080_bus.write_buffer(0x0000, [0xD3, 0x00].to_vec());                                // Stop.
    tst8080_bus.write_buffer(0x0005, [0xD3, 0x01, 0xC9].to_vec());                          // Print and ret.

    let mut cpu: Box<dyn CPU8080> = Box::new(Interpreter8080::new());
    cpu.force_jump(0x100);
    cpu.set_bus(tst8080_bus);
    cpu.run();

    let bus: Box<TestCPMBus> = (cpu.remove_bus() as Box<dyn Any>).downcast::<TestCPMBus>().unwrap();
    if bus.get_output() != "8080 Preliminary tests complete".as_bytes()
    {
        for char in bus.get_output()
        {
            print!("{}", *char as char);
        }
        panic!("[EROR]: Test ended with wrong output.");
    }
}

#[test]
fn test_8080exer_com()
{
    let mut tst8080_bus = Box::new(TestCPMBus::new());
    tst8080_bus.write_buffer(0x0100, read_file_to_vec("test_roms/8080EXER.COM"));

    // Write some patches for the ROM to actually do something in coordination with the sample Bus that is already implemented.
    tst8080_bus.write_buffer(0x0000, [0xD3, 0x00].to_vec());                                // Stop.
    tst8080_bus.write_buffer(0x0005, [0xD3, 0x01, 0xC9].to_vec());                          // Print and ret.

    let mut cpu: Box<dyn CPU8080> = Box::new(Interpreter8080::new());
    cpu.force_jump(0x100);
    cpu.set_bus(tst8080_bus);
    cpu.run();

    let bus: Box<TestCPMBus> = (cpu.remove_bus() as Box<dyn Any>).downcast::<TestCPMBus>().unwrap();
    if bus.get_output() != "idk".as_bytes()
    {
        for char in bus.get_output()
        {
            print!("{}", *char as char);
        }
        panic!("[EROR]: Test ended with wrong output.");
    }
}