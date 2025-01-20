use r8080::{cpu::{Register16, Registers}, Bus8080};

pub struct TestCPMBus
{
    ram: [u8; 0x10000],
    output: Vec<u8>
}

#[allow(dead_code)]
impl TestCPMBus
{
    pub fn new() -> Self {
        Self {
            ram: [0x00; 0x10000],
            output: Vec::new()
        }
    }

    pub fn dump_range(&self, start: u16, end: u16) {
        for (index, value) in self.ram[start as usize..end as usize].iter().enumerate() {
            println!("{:04X}: {:02X}", index + start as usize, value)
        }
    }

    pub fn get_output(&self) -> &Vec<u8> {
        &self.output
    }
}

impl Bus8080 for TestCPMBus
{
    fn in_b(&mut self, _: &mut Registers, _: u8) -> u8 {
        0xFF
    }

    fn out_b(&mut self, regs: &mut Registers, b: u8, a: u8) {
        match b {
            0x00 => {
                // Stop.
                regs.running = false
            }

            0x01 => {
                let operation = regs.c;
                match operation
                {
                    0x2 => {
                        // Print character in E.
                        self.output.push(regs.e);
                    }
                    0x9 => {
                        // Print string untill '$' is found.
                        let mut address = regs.get_16(&Register16::DE);
                        let mut character = self.read_b(address);
                        while self.read_b(address) != b'$'
                        {
                            self.output.push(character);
                            address += 1;
                            character = self.read_b(address);
                        }
                    }
                    _ => { panic!("Undefined operation on device 1: {:02X}", operation) }
                }
            }
            _ => { panic!("Out to unconnected device on port {:02X} with value {:02X}!", b, a) }
        }
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