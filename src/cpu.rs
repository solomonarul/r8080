mod instruction;
mod interpreter;

use std::{any::Any, sync::{Arc, RwLock}};

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

pub trait CPU8080: Any + Send + Sync
{
    fn get_executed_cycles(&mut self) -> u32;
    fn force_jump(&mut self, a: u16);
    fn set_bus(&mut self, b: Arc<RwLock<Box<dyn Bus8080>>>);
    fn get_bus(&self) -> Arc<RwLock<Box<dyn Bus8080>>>;
    fn stop(&mut self);
    fn is_running(&mut self) -> bool;
    fn step(&mut self);
    fn run(&mut self);
}