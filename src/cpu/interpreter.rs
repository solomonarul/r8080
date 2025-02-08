use std::sync::{Arc, RwLock};
use crate::{Bus8080, ErrorBus};
use crate::cpu::{CPU8080, Instruction8080, InstructionAction, Registers, Register16, Register8, RegisterFlags};

use super::instruction::InstructionTarget;
use super::Condition;

pub struct Interpreter8080
{
    cycles: u32,
    registers: Registers,
    bus: Arc<RwLock<Box<dyn Bus8080>>>
}

impl Interpreter8080
{
    pub fn new() -> Self {
        Self {
            cycles: 0x00,
            registers: Registers::new(),
            bus: Arc::new(RwLock::new(Box::new(ErrorBus::new())))
        }
    }
}

unsafe impl Sync for Interpreter8080{}
unsafe impl Send for Interpreter8080{}

impl CPU8080 for Interpreter8080
{
    fn get_executed_cycles(&mut self) -> u32 {
        self.cycles
    }

    fn force_jump(&mut self, a: u16) {
        self.registers.pc = a;
    }

    fn set_bus(&mut self, b: Arc<RwLock<Box<dyn Bus8080>>>) {
        self.bus = b;
    }

    fn get_bus(&self) -> Arc<RwLock<Box<dyn Bus8080>>> {
        Arc::clone(&self.bus)
    }

    fn stop(&mut self) {
        self.registers.running = false;
    }

    fn is_running(&mut self) -> bool {
        self.registers.running
    }

    fn step(&mut self) {
        let mut bus_write = self.bus.write().unwrap();
        // Check and execute interrupts if needed.
        let instruction = if self.registers.interrupts && bus_write.has_interrupt() {
            self.registers.halting = false;
            Instruction8080::from_opcode(bus_write.get_interrupt(), self.registers.pc, &bus_write)
        }
        else {
            if self.registers.halting { return }
            let opcode = bus_write.as_ref().read_b(self.registers.pc);
            let instruction = Instruction8080::from_opcode(opcode, self.registers.pc, &bus_write);
            self.registers.pc = self.registers.pc.wrapping_add(instruction.length as u16);
            instruction
        };

        self.cycles += match instruction.action {
        // NOP.
            InstructionAction::Nothing => { 4 }

        // Flow control section.
            InstructionAction::Jump { condition } => {
                if self.registers.check_condition(&condition) {
                    self.registers.pc = instruction.target.get_value_as_u16(&self.registers);
                }
                10
            }

            InstructionAction::Call { condition } => {
                let mut cycles = 11;
                if self.registers.check_condition(&condition) {
                    self.registers.sp = self.registers.sp.wrapping_sub(2);
                    bus_write.write_w(self.registers.sp, self.registers.pc);
                    self.registers.pc = instruction.target.get_value_as_u16(&self.registers);
                    cycles = 17;
                }
                if condition == Condition::None { cycles = 17; }
                cycles
            }

            InstructionAction::Return { condition } => {
                let mut cycles = 5;
                if self.registers.check_condition(&condition) {
                    self.registers.pc = bus_write.read_w(self.registers.sp);
                    self.registers.sp = self.registers.sp.wrapping_add(2);
                    cycles = 11;
                }
                if condition == Condition::None { cycles = 10; }
                cycles
            }

            InstructionAction::Halt => {
               self.registers.halting = true;
               7
            }

            InstructionAction::SetInterrupts { enabled } => {
                self.registers.interrupts = enabled;
                4
            }
        // End flow control section.

        // Carry section.
            InstructionAction::SetCarry { value } => {
                self.registers.set_flag(RegisterFlags::Carry, value);
                4
            }

            InstructionAction::ComplementCarry => {
                self.registers.set_flag(RegisterFlags::Carry, !self.registers.get_flag(RegisterFlags::Carry));
                4
            }
        // End carry section.
        
        // 8-bit registers section.
            InstructionAction::MovReg { register } => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                self.registers.set_8(&register, &mut bus_write, value);
                if matches!(instruction.target, InstructionTarget::Immediate8 { .. }) {
                    if register == Register8::M { 10 } else { 7 }
                } else {
                    if register == Register8::M { 7 } else { 5 }
                }
            }

            InstructionAction::IncrementReg { register } => {
                let register_value = self.registers.get_8(&mut bus_write, &register) as u16;
                let result = ((register_value + 1) & 0xFF) as u8;

                // Set flags.
                self.registers.set_flag(RegisterFlags::HalfCarry, (result & 0xF) == 0x0);
                self.registers.set_zsp(result);

                self.registers.set_8(&register, &mut bus_write, result);
                if register == Register8::M { 10 } else { 5 }
            }

            InstructionAction::DecrementReg { register } => {
                let register_value = self.registers.get_8(&mut bus_write, &register);
                let result = register_value.wrapping_sub(1);
                
                // Set flags.
                self.registers.set_flag(RegisterFlags::HalfCarry, (result & 0xF) != 0xF);
                self.registers.set_zsp(result);

                self.registers.set_8(&register, &mut bus_write, result);
                if register == Register8::M { 10 } else { 5 }
            }       

            InstructionAction::AddReg { register, carry } => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers) as u16;
                let register_value = self.registers.get_8(&mut bus_write, &register) as u16;
                let carry = if carry && self.registers.get_flag(RegisterFlags::Carry) { 1 } else { 0 };
                let result = register_value + value + carry;
                
                // Set flags.
                self.registers.set_flag(RegisterFlags::Carry, ((result ^ register_value ^ value) & (1 << 8)) != 0);
                self.registers.set_flag(RegisterFlags::HalfCarry, ((result ^ register_value ^ value) & (1 << 4)) != 0);
                
                let result = (result & 0xFF) as u8;
                self.registers.set_zsp(result);
                self.registers.set_8(&register, &mut bus_write, result);
                if instruction.target == (InstructionTarget::Register8 { register: Register8::M }) { 7 } else { 4 }
            }

            InstructionAction::SubReg { register, borrow: carry } => {
                // Subtraction is same as addition with !value and inverted carries.
                let value = !(instruction.target.get_value_as_u8(&mut bus_write, &self.registers) as u16);
                let register_value = self.registers.get_8(&mut bus_write, &register) as u16;
                let carry = if carry && self.registers.get_flag(RegisterFlags::Carry) { 0 } else { 1 };
                let result = register_value.wrapping_add(value).wrapping_add(carry);
                
                // Set flags.
                self.registers.set_flag(RegisterFlags::Carry, ((result ^ register_value ^ value) & (1 << 8)) == 0);
                self.registers.set_flag(RegisterFlags::HalfCarry, ((result ^ register_value ^ value) & (1 << 4)) != 0);
                
                let result = (result & 0xFF) as u8;
                self.registers.set_zsp(result);
                self.registers.set_8(&register, &mut bus_write, result);
                if instruction.target == (InstructionTarget::Register8 { register: Register8::M }) { 7 } else { 4 }
            }

            InstructionAction::CompareReg { register } => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers) as u16;
                let register_value = self.registers.get_8(&mut bus_write, &register) as u16;
                let result = register_value.wrapping_sub(value);
                
                self.registers.set_flag(RegisterFlags::Carry, (result >> 8) != 0);
                self.registers.set_flag(RegisterFlags::HalfCarry,  !(register_value ^ result ^ value) & 0x10 != 0);
                self.registers.set_zsp((result & 0xFF) as u8);
                if instruction.target == (InstructionTarget::Register8 { register: Register8::M }) { 7 } else { 4 }
            }

            InstructionAction::AndReg { register } => {
                let register_value = self.registers.get_8(&mut bus_write, &register);
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let result = register_value & value;

                self.registers.set_flag(RegisterFlags::Carry, false);
                self.registers.set_flag(RegisterFlags::HalfCarry, (register_value | value) & 0x08 != 0);
                self.registers.set_zsp(result);

                self.registers.set_8(&register, &mut bus_write, result);
                if instruction.target == (InstructionTarget::Register8 { register: Register8::M }) { 7 } else { 4 }
            }

            InstructionAction::OrReg { register } => {
                let register_value = self.registers.get_8(&mut bus_write, &register);
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let result = register_value | value;

                self.registers.set_flag(RegisterFlags::Carry, false);
                self.registers.set_flag(RegisterFlags::HalfCarry, false);
                self.registers.set_zsp(result);

                self.registers.set_8(&register, &mut bus_write, result);
                if instruction.target == (InstructionTarget::Register8 { register: Register8::M }) { 7 } else { 4 }
            }

            InstructionAction::XorReg { register } => {
                let register_value = self.registers.get_8(&mut bus_write, &register);
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let result = register_value ^ value;

                self.registers.set_flag(RegisterFlags::Carry, false);
                self.registers.set_flag(RegisterFlags::HalfCarry, false);
                self.registers.set_zsp(result);
                
                self.registers.set_8(&Register8::A, &mut bus_write, result);
                if instruction.target == (InstructionTarget::Register8 { register: Register8::M }) { 7 } else { 4 }
            }

            InstructionAction::ComplementReg { register } => {
                let mut value = self.registers.get_8(&mut bus_write, &register);
                value = !value;
                self.registers.set_8(&register, &mut bus_write, value);
                4
            }

            InstructionAction::StoreRegToMemory { register } => {
                let value = self.registers.get_8(&mut bus_write, &register);
                let location = instruction.target.get_value_as_u16(&self.registers);
                bus_write.write_b(location, value);
                if instruction.target == (InstructionTarget::Register16 { register: Register16::HL }) { 16 } else { 10 }
            }

            InstructionAction::LoadRegFromMemory { register } => {
                let location = instruction.target.get_value_as_u16(&self.registers);
                let value = bus_write.read_b(location);
                self.registers.set_8(&register, &mut bus_write, value);
                if matches!(instruction.target, InstructionTarget::Register16 { .. }) { 16 } else { 13 }
            }

            InstructionAction::DAAReg { register } => {
                let register_value = self.registers.get_8(&mut bus_write, &register) as u16;
                let mut carry = self.registers.get_flag(RegisterFlags::Carry);
                let mut correction = 0x00;

                let lsb = register_value & 0xF;
                let msb = register_value >> 4;

                if self.registers.get_flag(RegisterFlags::HalfCarry) || lsb > 9 {
                    correction += 0x06;
                }

                if self.registers.get_flag(RegisterFlags::Carry) || msb > 9 || (msb >= 9 && lsb > 9) {
                    correction += 0x60;
                    carry = true;
                }

                // Set flags.
                let result = register_value + correction;
                self.registers.set_flag(RegisterFlags::Carry, ((result ^ register_value ^ correction) & (1 << 8)) != 0);
                self.registers.set_flag(RegisterFlags::HalfCarry, ((result ^ register_value ^ correction) & (1 << 4)) != 0);
                
                let result = (result & 0xFF) as u8;
                self.registers.set_zsp(result);
                self.registers.set_8(&register, &mut bus_write, result);
                self.registers.set_flag(RegisterFlags::Carry, carry);
                4
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
                4
            }
        // End 8-bit registers section.

        // 16-bit registers section.
            InstructionAction::Load16 { ref register} => {
                self.registers.set_16(register, instruction.target.get_value_as_u16(&self.registers));
                10
            }

            InstructionAction::Increment16 { register } => {
                let value = self.registers.get_16(&register).wrapping_add(1);
                self.registers.set_16(&register, value);
                5
            }

            InstructionAction::Decrement16 { register } => {
                let value = self.registers.get_16(&register).wrapping_sub(1);
                self.registers.set_16(&register, value);
                5
            }

            InstructionAction::Add16 { register } => {
                let register_value = self.registers.get_16(&register);
                let value = instruction.target.get_value_as_u16(&mut self.registers);
                let (result, carry) = register_value.overflowing_add(value);
                self.registers.set_flag(RegisterFlags::Carry, carry);
                self.registers.set_16(&register, result);
                10
            }

            InstructionAction::Push16 { ref register} => {
                let value = self.registers.get_16(register);
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                bus_write.write_w(self.registers.sp, value);
                11
            }

            InstructionAction::Pop16 { ref register} => {
                let value = bus_write.read_w(self.registers.sp);
                self.registers.sp = self.registers.sp.wrapping_add(2);
                self.registers.set_16(register, value);
                10
            }

            InstructionAction::LoadReg16FromMemory { register } => {
                let location = instruction.target.get_value_as_u16(&self.registers);
                let value = bus_write.read_w(location);
                self.registers.set_16(&register, value);
                16
            }

            InstructionAction::StoreReg16ToMemory { register } => {
                let value = self.registers.get_16(&register);
                let location = instruction.target.get_value_as_u16(&self.registers);
                bus_write.write_w(location, value);
                16
            }

            InstructionAction::Exchange => {
                let hl = self.registers.get_16(&Register16::HL);
                let de = self.registers.get_16(&Register16::DE);
                self.registers.set_16(&Register16::DE, hl);
                self.registers.set_16(&Register16::HL, de);
                5
            }

            InstructionAction::ExchangeToStack => {
                let value = bus_write.read_w(self.registers.sp);
                let hl = self.registers.get_16(&Register16::HL);
                bus_write.write_w(self.registers.sp, hl);
                self.registers.set_16(&Register16::HL, value);
                18
            }
        // End of 16-bit registers section.

        // Bus section.
            InstructionAction::In8 => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                self.registers.a = bus_write.in_b(&mut self.registers, value);
                10
            }

            InstructionAction::Out8 => {
                let value = instruction.target.get_value_as_u8(&mut bus_write, &self.registers);
                let a = self.registers.a;
                bus_write.out_b(&mut self.registers, value, a);
                10
            }
        // End of bus section.

        // Default / unimplemented.
            _ => {
                self.registers.running = false;
                println!(
                    "[EROR]: Unknown opcode found at PC 0x{:04X}: 0x{:02X}",
                    self.registers.pc - instruction.length as u16, instruction.opcode
                );
                println!("{:#?}", instruction);
                panic!("[WARN]: Dying...");  // TODO: do some actual error handling instead of dying.
            }
        }        
    }

    fn run(&mut self) {
        while self.registers.running {
            self.step();
        }
    }
}
