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
    let mut bus = Box::new(TestCPMBus::new("\x00\x00\x00\x00\x00\x00\x0D\x0ADIAGNOSTICS II V1.2 - CPU TEST\x0D\x0ACOPYRIGHT (C) 1981 - SUPERSOFT ASSOCIATES\x0D\x0A\x0AABCDEFGHIJKLMNOPQRSTUVWXYZ\x0D\x0ACPU IS 8080/8085\x0D\x0ABEGIN TIMING TEST\x0D\x0A\x07\x07END TIMING TEST\x0D\x0ACPU TESTS OK\x0D\x0A"));
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
fn test_8080exm_com()
{
    let mut bus = Box::new(TestCPMBus::new("8080 instruction exerciser\x0A\x0Ddad <b,d,h,sp>................  PASS! crc is:14474ba6\x0A\x0Daluop nn......................  PASS! crc is:9e922f9e\x0A\x0Daluop <b,c,d,e,h,l,m,a>.......  PASS! crc is:cf762c86\x0A\x0D<daa,cma,stc,cmc>.............  PASS! crc is:bb3f030c\x0A\x0D<inr,dcr> a...................  PASS! crc is:adb6460e\x0A\x0D<inr,dcr> b...................  PASS! crc is:83ed1345\x0A\x0D<inx,dcx> b...................  PASS! crc is:f79287cd\x0A\x0D<inr,dcr> c...................  PASS! crc is:e5f6721b\x0A\x0D<inr,dcr> d...................  PASS! crc is:15b5579a\x0A\x0D<inx,dcx> d...................  PASS! crc is:7f4e2501\x0A\x0D<inr,dcr> e...................  PASS! crc is:cf2ab396\x0A\x0D<inr,dcr> h...................  PASS! crc is:12b2952c\x0A\x0D<inx,dcx> h...................  PASS! crc is:9f2b23c0\x0A\x0D<inr,dcr> l...................  PASS! crc is:ff57d356\x0A\x0D<inr,dcr> m...................  PASS! crc is:92e963bd\x0A\x0D<inx,dcx> sp..................  PASS! crc is:d5702fab\x0A\x0Dlhld nnnn.....................  PASS! crc is:a9c3d5cb\x0A\x0Dshld nnnn.....................  PASS! crc is:e8864f26\x0A\x0Dlxi <b,d,h,sp>,nnnn...........  PASS! crc is:fcf46e12\x0A\x0Dldax <b,d>....................  PASS! crc is:2b821d5f\x0A\x0Dmvi <b,c,d,e,h,l,m,a>,nn......  PASS! crc is:eaa72044\x0A\x0Dmov <bcdehla>,<bcdehla>.......  PASS! crc is:10b58cee\x0A\x0Dsta nnnn / lda nnnn...........  PASS! crc is:ed57af72\x0A\x0D<rlc,rrc,ral,rar>.............  PASS! crc is:e0d89235\x0A\x0Dstax <b,d>....................  PASS! crc is:2b0471e9\x0A\x0DTests complete"));
    bus.write_buffer(0x0100, read_file_to_vec("test_roms/8080EXM.COM"));

    let mut cpu = Box::new(Interpreter8080::new()) as Box<dyn CPU8080>;
    cpu.force_jump(0x100);
    cpu.set_bus(Arc::new(RwLock::new(bus)));
    cpu.run();
}
