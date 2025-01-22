use r8080::{cpu::Registers, Bus8080};

// Simple Bus that is mostly used for testing.
pub struct EchoBus
{
    ram: [u8; 0x10000]
}

#[allow(dead_code)]
impl EchoBus
{
    pub fn new() -> Self {
        Self {
            ram: [0x00; 0x10000]
        }
    }

    pub fn dump_range(&self, start: u16, end: u16) {
        for (index, value) in self.ram[start as usize..end as usize].iter().enumerate() {
            println!("{:04X}: {:02X}", index + start as usize, value)
        }
    }
}

impl Bus8080 for EchoBus
{
    fn in_b(&mut self, _: &mut Registers, b: u8) -> u8 {
        println!("[INFO]: Read 0xFF from device {:02X} on EchoBus.", b);
        0xFF
    }

    fn out_b(&mut self, _: &mut Registers, b: u8, a: u8) {
        println!("[INFO]: Written {:02X} to device {:02X} on EchoBus.", a, b);
    }

    fn read_b(&self, a: u16) -> u8 {
        return self.ram[a as usize];
    }

    fn read_w(&self, a: u16) -> u16 {
        return ((self.read_b(a + 1) as u16) << 8)  | self.read_b(a) as u16;
    }

    fn write_b(&mut self, a: u16, b: u8) {
        self.ram[a as usize] = b;
    }

    fn write_w(&mut self, a: u16, w: u16) {
        self.write_b(a + 1, (w >> 8) as u8);
        self.write_b(a, (w & 0xFF) as u8);
    }

    fn write_buffer(&mut self, a: u16, data: Vec<u8>) {
        self.ram[a as usize..a as usize + data.len()].copy_from_slice(data.as_slice());
    }
}