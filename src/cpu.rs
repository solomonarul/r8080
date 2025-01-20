mod instruction;
mod interpreter;

use crate::Bus8080;

pub type Interpreter8080 = interpreter::Interpreter8080;
pub type Registers = instruction::Registers;
pub type Register8 = instruction::Register8;
pub type Register16 = instruction::Register16;
pub type Instruction8080 = instruction::Instruction8080;
pub type InstructionAction = instruction::InstructionAction;
pub type InstructionType = instruction::InstructionTarget;
pub type Condition = instruction::Condition;
pub type RegisterFlags = instruction::RegisterFlags;

pub trait CPU8080
{
    fn force_jump(&mut self, a: u16);
    fn set_bus(&mut self, b: Box<dyn Bus8080>);
    fn remove_bus(&mut self) -> Box<dyn Bus8080>;
    fn run(&mut self);
}