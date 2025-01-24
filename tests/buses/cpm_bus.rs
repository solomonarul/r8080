use r8080::{cpu::{Register16, Registers}, Bus8080};

pub struct TestCPMBus
{
    ram: [u8; 0x10000],
    output: Vec<u8>,
    expected_output: &'static str
}

impl TestCPMBus
{
    pub fn new(expected_output: &'static str) -> Self {
        let mut result = Self {
            ram: [0x00; 0x10000],
            output: Vec::new(),
            expected_output
        };

        result.write_buffer(0x0000, [0xD3, 0x00].to_vec());         // Stop.
        result.write_buffer(0x0005, [0xD3, 0x01, 0xC9].to_vec());   // Print and ret.

        result
    }
}

impl Drop for TestCPMBus {
    fn drop(&mut self) {
        // If output is not what we expected, cry about it.
        if self.output != self.expected_output.as_bytes() {
            for char in &self.output
            {
                print!("{}", *char as char);
            }
            panic!("[EROR]: Test ended with wrong output.");
        }
    }
}

impl Bus8080 for TestCPMBus
{
    fn get_interrupt(&mut self) -> u8 {
        0x00
    }
    
    fn has_interrupt(&self) -> bool {
        false
    }

    fn push_interrupt(&mut self, _: u8) {
        
    }

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