use std::mem::replace;
use std::sync::{Arc, RwLock};
use crate::{Bus8080, ErrorBus};
use crate::cpu::{CPU8080, Instruction8080, InstructionAction, Registers, Register16, Register8, RegisterFlags};

pub struct Interpreter8080
{
    registers: Registers,
    bus: Arc<RwLock<Box<dyn Bus8080>>>
}

impl Interpreter8080
{
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            bus: Arc::new(RwLock::new(Box::new(ErrorBus::new())))
        }
    }
}

unsafe impl Sync for Interpreter8080{}
unsafe impl Send for Interpreter8080{}

impl CPU8080 for Interpreter8080
{
    fn force_jump(&mut self, a: u16) {
        self.registers.pc = a;
    }

    fn set_bus(&mut self, b: Arc<RwLock<Box<dyn Bus8080>>>) {
        self.bus = b;
    }

    fn get_bus(&self) -> Arc<RwLock<Box<dyn Bus8080>>> {
        Arc::clone(&self.bus)
    }

    fn remove_bus(&mut self) -> Arc<RwLock<Box<dyn Bus8080>>> {
        replace(&mut self.bus, Arc::new(RwLock::new(Box::new(ErrorBus::new()))))
    }

    fn stop(&mut self) {
        self.registers.running = false;
    }

    fn is_running(&mut self) -> bool {
        self.registers.running
    }

    fn step(&mut self) {
        let mut bus_write = self.bus.write().unwrap();
        let instruction = Instruction8080::from_opcode(self.registers.pc, &bus_write);
        self.registers.pc += instruction.length as u16;

        match instruction.action {
        // Default / unimplemented.
            InstructionAction::None => {
                self.registers.running = false;
                println!(
                    "[EROR]: Unknown opcode found at PC 0x{:04X}: 0x{:02X}",
                    self.registers.pc - instruction.length as u16, instruction.opcode
                );
                dbg!(&instruction);
                panic!("[WARN]: Dying...");  // TODO: do some actual error handling instead of dying.
            }

        // NOP.
            InstructionAction::Nothing => {}

        // Flow control section.
            InstructionAction::Jump { condition } => {
                if self.registers.check_condition(&condition) {
                    self.registers.pc = instruction.target.get_value_as_u16(&self.registers);
                }
            }

            InstructionAction::Call { condition } => {
                if self.registers.check_condition(&condition) {
                    self.registers.sp = self.registers.sp.wrapping_sub(2);
                    bus_write.write_w(self.registers.sp, self.registers.pc);
                    self.registers.pc = instruction.target.get_value_as_u16(&self.registers);
                }
            }

            InstructionAction::Return { condition } => {
                if self.registers.check_condition(&condition) {
                    self.registers.pc = bus_write.read_w(self.registers.sp);
                    self.registers.sp = self.registers.sp.wrapping_add(2);
                }
            }

            InstructionAction::Halt => {
                self.registers.running = false; // TODO: actual halting.
            }

            // TODO: proper interrupt handling.
            InstructionAction::SetInterrupts { enabled } => {
                self.registers.interrupts = enabled
            }
        // End flow control section.

        // Carry section.
            InstructionAction::SetCarry { value } => {
                self.registers.set_flag(RegisterFlags::Carry, value);
            }

            InstructionAction::ComplementCarry => {
                self.registers.set_flag(RegisterFlags::Carry, !self.registers.get_flag(RegisterFlags::Carry));
            }
        // End carry section.
        
        // 8-bit registers section.
            InstructionAction::MovReg { register } => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                self.registers.set_8(&register, &mut bus_write, value);
            }

            InstructionAction::AddReg { register, carry } => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let register_value = self.registers.get_8(&mut bus_write, &register);
                
                // Do the addition with carry if needed.
                let mut half_carry = (register_value & 0xF) + (value & 0xF) > 0xF;
                let (mut result, mut carry_out) = register_value.overflowing_add(value);

                if carry {
                    let carry_value = if self.registers.get_flag(RegisterFlags::Carry) { 1 } else { 0 };
                    half_carry = ((result & 0xF) + (carry_value & 0xF) > 0xF) || half_carry;
                    let (result_carry, carry_out_carry) = result.overflowing_add(carry_value);
                    result = result_carry;
                    carry_out = carry_out_carry || carry_out;
                }
                
                // Set flags.
                self.registers.set_flag(RegisterFlags::Zero, result == 0);
                self.registers.set_flag(RegisterFlags::Sign, result & 0x80 != 0);
                self.registers.set_flag(RegisterFlags::Parity, result.count_ones() % 2 == 0);
                self.registers.set_flag(RegisterFlags::HalfCarry, half_carry);
                self.registers.set_flag(RegisterFlags::Carry, carry_out);

                self.registers.set_8(&register, &mut bus_write, result);
            }

            InstructionAction::SubReg { register, carry } => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let register_value = self.registers.get_8(&mut bus_write, &register);
                
                // Do the addition with carry if needed.
                let mut half_carry = (register_value & 0xF) < (value & 0xF);
                let (mut result, mut carry_out) = register_value.overflowing_sub(value);

                if carry {
                    let carry_value = if self.registers.get_flag(RegisterFlags::Carry) { 1 } else { 0 };
                    half_carry = ((result & 0xF) < (carry_value & 0xF)) || half_carry;
                    let (result_carry, carry_out_carry) = result.overflowing_sub(carry_value);
                    result = result_carry;
                    carry_out = carry_out_carry || carry_out;
                }
                
                // Set flags.
                self.registers.set_flag(RegisterFlags::Zero, result == 0);
                self.registers.set_flag(RegisterFlags::Sign, result & 0x80 != 0);
                self.registers.set_flag(RegisterFlags::Parity, result.count_ones() % 2 == 0);
                self.registers.set_flag(RegisterFlags::HalfCarry, half_carry);
                self.registers.set_flag(RegisterFlags::Carry, carry_out);

                self.registers.set_8(&register, &mut bus_write, result);
            }

            InstructionAction::CompareReg { register } => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let register_value = self.registers.get_8(&mut bus_write, &register);
                
                // Do the addition with carry if needed.
                let half_carry = (register_value & 0xF) < (value & 0xF);
                let (result, carry_out) = register_value.overflowing_sub(value);
                
                self.registers.set_flag(RegisterFlags::Zero, result == 0);
                self.registers.set_flag(RegisterFlags::Sign, result & 0x80 != 0);
                self.registers.set_flag(RegisterFlags::Parity, result.count_ones() % 2 == 0);
                self.registers.set_flag(RegisterFlags::HalfCarry, half_carry);
                self.registers.set_flag(RegisterFlags::Carry, carry_out);
            }

            InstructionAction::AndReg { register } => {
                let register = self.registers.get_8(&mut bus_write, &register);
                let target = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let result = register & target;

                self.registers.set_flag(RegisterFlags::Zero, result == 0);
                self.registers.set_flag(RegisterFlags::Sign, result & 0x80 != 0);
                self.registers.set_flag(RegisterFlags::Parity, result.count_ones() % 2 == 0);

                self.registers.set_8(&Register8::A, &mut bus_write, result);
            }

            InstructionAction::OrReg { register } => {
                let register = self.registers.get_8(&mut bus_write, &register);
                let target = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let result = register | target;

                self.registers.set_flag(RegisterFlags::Zero, result == 0);
                self.registers.set_flag(RegisterFlags::Sign, result & 0x80 != 0);
                self.registers.set_flag(RegisterFlags::Parity, result.count_ones() % 2 == 0);

                self.registers.set_8(&Register8::A, &mut bus_write, result);
            }

            InstructionAction::XorReg { register } => {
                let register = self.registers.get_8(&mut bus_write, &register);
                let target = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let result = register ^ target;

                self.registers.set_flag(RegisterFlags::Zero, result == 0);
                self.registers.set_flag(RegisterFlags::Sign, result & 0x80 != 0);
                self.registers.set_flag(RegisterFlags::Parity, result.count_ones() % 2 == 0);

                self.registers.set_8(&Register8::A, &mut bus_write, result);
            }

            InstructionAction::ComplementReg { register } => {
                let mut value = self.registers.get_8(&mut bus_write, &register);
                value = !value;
                self.registers.set_8(&register, &mut bus_write, value);
            }

            InstructionAction::StoreRegToMemory { register } => {
                let value = self.registers.get_8(&mut bus_write, &register);
                let location = instruction.target.get_value_as_u16(&self.registers);
                bus_write.write_b(location, value);
            }

            InstructionAction::LoadRegFromMemory { register } => {
                let location = instruction.target.get_value_as_u16(&self.registers);
                let value = bus_write.read_b(location);
                self.registers.set_8(&register, &mut bus_write, value);
            }

            InstructionAction::DAAReg { register } => {
                let register_value = self.registers.get_8(&mut bus_write, &register);
                let mut value = 0x00;
            
                if register_value & 0x0F > 9 || self.registers.get_flag(RegisterFlags::HalfCarry) {
                    value += 0x06;
                }
                if register_value > 0x99 || self.registers.get_flag(RegisterFlags::Carry) { 
                    value += 0x60;
                }
                
                let half_carry = (register_value & 0xF) + (value & 0xF) > 0xF;
                let (result, carry_out) = register_value.overflowing_add(value);
                self.registers.set_flag(RegisterFlags::Zero, result == 0);
                self.registers.set_flag(RegisterFlags::Sign, result & 0x80 != 0);
                self.registers.set_flag(RegisterFlags::Parity, result.count_ones() % 2 == 0);
                self.registers.set_flag(RegisterFlags::HalfCarry, half_carry);
                self.registers.set_flag(RegisterFlags::Carry, carry_out);
            
                self.registers.set_8(&register, &mut bus_write, result);
            }
            
            InstructionAction::RotateReg { register, right, arithmetic } => {
                let mut value = self.registers.get_8(&mut bus_write, &register);
                let carry_in = if self.registers.get_flag(RegisterFlags::Carry) { 1 } else { 0 };

                let (result, carry_out) = if !arithmetic {
                    if right {
                        let carry_out = value & 1;
                        value = value >> 1;
                        value |= carry_out << 7;
                        (value, carry_out)
                    } else {
                        let carry_out = (value & 0x80) >> 7;
                        value = value << 1;
                        value |= carry_out;
                        (value, carry_out)
                    }
                }
                else {
                    if right {
                        let carry_out = value & 1;
                        value = value >> 1;
                        value |= carry_in << 7;
                        (value, carry_out)
                    } else {
                        let carry_out = (value & 0x80) >> 7;
                        value = value << 1;
                        value |= carry_in;
                        (value, carry_out)
                    }                        
                };

                self.registers.set_flag(RegisterFlags::Carry, carry_out != 0);

                self.registers.set_8(&register, &mut bus_write, result);
            }
        // End 8-bit registers section.

        // 16-bit registers section.
            InstructionAction::Load16 { ref register} => {
                self.registers.set_16(register, instruction.target.get_value_as_u16(&self.registers));
            }

            InstructionAction::Increment16 { register } => {
                let value = self.registers.get_16(&register);
                self.registers.set_16(&register, value + 1);
            }

            InstructionAction::Decrement16 { register } => {
                let value = self.registers.get_16(&register);
                self.registers.set_16(&register, value - 1);
            }

            InstructionAction::Add16 { register } => {
                let value = instruction.target.get_value_as_u16(&mut self.registers);
                let register_value = self.registers.get_16(&register);
                let (result, carry) = register_value.overflowing_add(value);
                self.registers.set_flag(RegisterFlags::Carry, carry);
                self.registers.set_16(&register, result);
            }

            InstructionAction::Push16 { ref register} => {
                let value = self.registers.get_16(register);
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                bus_write.write_w(self.registers.sp, value);
            }

            InstructionAction::Pop16 { ref register} => {
                let value = bus_write.read_w(self.registers.sp);
                self.registers.sp = self.registers.sp.wrapping_add(2);
                self.registers.set_16(register, value);
            }

            InstructionAction::LoadReg16FromMemory { register } => {
                let location = instruction.target.get_value_as_u16(&self.registers);
                let value = bus_write.read_w(location);
                self.registers.set_16(&register, value);
            }

            InstructionAction::StoreReg16ToMemory { register } => {
                let value = self.registers.get_16(&register);
                let location = instruction.target.get_value_as_u16(&self.registers);
                bus_write.write_w(location, value);
            }

            InstructionAction::Exchange => {
                let hl = self.registers.get_16(&Register16::HL);
                let de = self.registers.get_16(&Register16::DE);
                self.registers.set_16(&Register16::DE, hl);
                self.registers.set_16(&Register16::HL, de);
            }

            InstructionAction::ExchangeToStack => {
                let hl = self.registers.get_16(&Register16::HL);
                let value = bus_write.read_w(self.registers.sp);
                bus_write.write_w(self.registers.sp, hl);
                self.registers.set_16(&Register16::HL, value);
            }
        // End of 16-bit registers section.

        // Bus section.
            InstructionAction::In8 => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                self.registers.a = bus_write.in_b(&mut self.registers, value);
            }

            InstructionAction::Out8 => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let a = self.registers.a;
                bus_write.out_b(&mut self.registers, value, a);
            }
        // End of bus section.
        }        
    }

    fn run(&mut self) {
        while self.registers.running {
            self.step();
        }
    }
}
