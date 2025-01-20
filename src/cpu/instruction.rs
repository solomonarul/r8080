use crate::Bus8080;

#[derive(Debug, Clone, Copy)]
pub enum Condition
{
    None,
    NotZero, Zero,
    NotCarry, Carry,
    Auxiliary, NotAuxiliary,
    PairtyOdd, ParityEven,
    Plus, Minus
}

#[derive(Debug, Clone, Copy)]
pub enum Register16
{
    BC, DE, HL, SP, DSW
}

#[derive(Debug, Clone, Copy)]
pub enum Register8
{
    A, B, C, D, E, F, H, L, M
}

#[derive(Debug)]
pub enum InstructionAction
{
    None,
    Nothing,
    Jump { condition: Condition },
    Call { condition: Condition },
    Return { condition: Condition },
    Increment16 { register: Register16 },
    Decrement16 { register: Register16 },
    Add16 { register: Register16 },
    Load16 { register: Register16 },
    Push16 { register: Register16 },
    Pop16 { register: Register16 },
    AndReg { register: Register8 },
    OrReg { register: Register8 },
    MovReg { register: Register8 },
    XorReg { register: Register8 },
    AddReg { register: Register8, carry: bool },
    SubReg { register: Register8, carry: bool },
    DAAReg { register: Register8 },
    RotateReg { register: Register8, right: bool, arithmetic: bool },
    CompareReg { register: Register8 },
    ComplementReg { register: Register8 },
    StoreRegToMemory { register: Register8 },
    LoadRegFromMemory { register: Register8 },
    StoreReg16ToMemory { register: Register16 },
    LoadReg16FromMemory { register: Register16 },
    SetInterrupts { enabled: bool },
    SetCarry { value: bool },
    ComplementCarry,
    ExchangeToStack,
    Exchange,
    Halt,
    In8,
    Out8,
}

#[derive(Debug)]
pub enum RegisterFlags
{
    Carry,
    HalfCarry,
    Sign,
    Zero,
    Parity
}

#[derive(Debug)]
pub enum InstructionTarget
{
    None,
    Register8 { register: Register8 },
    Register16 { register: Register16 },
    Immediate8 { value: u8 },
    Immediate16 { value: u16 },
}

impl InstructionTarget
{
    pub fn get_value_as_u16(&self, registers: &Registers) -> u16
    {
        match self {
            Self::Immediate16 { value } => *value,
            Self::Register16 { register } => registers.get_16(register),
            _ => panic!("[EROR]: Unimplemented data source in get_value_as_u16!")
        }
    }

    pub fn get_value_as_u8(&self, bus: &mut Box<dyn Bus8080>, registers: &Registers) -> u8
    {
        match self {
            Self::Immediate8 { value } => *value,
            Self::Register8 { register } => { registers.get_8(bus, register)  }
            _ => panic!("[EROR]: Unimplemented data source in get_value_as_u8!")
        }
    }
}

#[derive(Debug)]
pub struct Registers
{
    pub pc: u16,
    pub sp: u16,
    pub h: u8, pub l: u8,
    pub a: u8, pub b: u8, pub c: u8,
    pub d: u8, pub e: u8, pub f: u8,
    pub interrupts: bool,
    pub running: bool
}

impl Registers
{
    pub fn new() -> Self {
        Self {
            pc: 0x0000,
            sp: 0x0000,
            h: 0x00, l: 0x00,
            a: 0x00, b: 0x00, c: 0x00,
            d: 0x00, e: 0x00, f: 0x02,
            interrupts: true, running: true
        }
    }

    pub fn set_flag(&mut self, flag: RegisterFlags, value: bool)
    {
        self.f = match flag
        {
            RegisterFlags::Carry => {
                if !value { self.f & !(1 << 0) }
                else { self.f | (1 << 0) }
            }
            RegisterFlags::Parity => {
                if !value { self.f & !(1 << 2) }
                else { self.f | (1 << 2) }
            }
            RegisterFlags::HalfCarry => {
                if !value { self.f & !(1 << 4) }
                else { self.f | (1 << 4) }
            }
            RegisterFlags::Zero => {
                if !value { self.f & !(1 << 6) }
                else { self.f | (1 << 6) }
            }
            RegisterFlags::Sign => {
                if !value { self.f & !(1 << 7) }
                else { self.f | (1 << 7) }
            }
        }
    }
    
    pub fn get_flag(&self, flag: RegisterFlags) -> bool {
        match flag
        {
            RegisterFlags::Carry => { self.f & (1 << 0) != 0 }
            RegisterFlags::Parity => { self.f & (1 << 2) != 0 }
            RegisterFlags::HalfCarry => { self.f & (1 << 4) != 0 }
            RegisterFlags::Zero => { self.f & (1 << 6) != 0 }
            RegisterFlags::Sign => { self.f & (1 << 7) != 0 }
        }
    }

    pub fn check_condition(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Carry => { self.get_flag(RegisterFlags::Carry) }
            Condition::NotCarry => { !self.get_flag(RegisterFlags::Carry) }
            Condition::PairtyOdd => { !self.get_flag(RegisterFlags::Parity) }
            Condition::ParityEven => { self.get_flag(RegisterFlags::Parity) }
            Condition::Auxiliary => { self.get_flag(RegisterFlags::HalfCarry) }
            Condition::NotAuxiliary => { !self.get_flag(RegisterFlags::HalfCarry) }
            Condition::Zero => { self.get_flag(RegisterFlags::Zero) }
            Condition::NotZero => { !self.get_flag(RegisterFlags::Zero) }
            Condition::Plus => { !self.get_flag(RegisterFlags::Sign) }
            Condition::Minus => { self.get_flag(RegisterFlags::Sign) }
            Condition::None => { true }
        }
    }
    

    pub fn get_16(&self, register: &Register16) -> u16 {
        match register {
            Register16::BC => { ((self.b as u16) << 8) as u16 | self.c as u16 }
            Register16::DE => { ((self.d as u16) << 8) as u16 | self.e as u16 }
            Register16::HL => { ((self.h as u16) << 8) as u16 | self.l as u16 }
            Register16::DSW => { ((self.a as u16) << 8) as u16 | self.f as u16 }
            Register16::SP => { self.sp }
        }
    }

    pub fn get_8(&self, bus: &mut Box<dyn Bus8080>, register: &Register8) -> u8 {
        match register {
            Register8::A => { self.a }
            Register8::B => { self.b }
            Register8::C => { self.c }
            Register8::D => { self.d }
            Register8::E => { self.e }
            Register8::F => { self.f }
            Register8::H => { self.h }
            Register8::L => { self.l }
            Register8::M => { bus.read_b(self.get_16(&Register16::HL)) }
        }
    }

    pub fn set_16(&mut self, register: &Register16, value: u16) {
        match register {
            Register16::BC => {
                self.b = (value >> 8) as u8;
                self.c = (value & 0xFF) as u8;
            }
            Register16::DE => {
                self.d = (value >> 8) as u8;
                self.e = (value & 0xFF) as u8;
            }
            Register16::HL => {
                self.h = (value >> 8) as u8;
                self.l = (value & 0xFF) as u8;
            }
            Register16::DSW => {
                self.a = (value >> 8) as u8;
                self.f = (value & 0xFF) as u8;
            }
            Register16::SP => { self.sp = value; }
        }
    }

    pub fn set_8(&mut self, register: &Register8, bus: &mut Box<dyn Bus8080>, value: u8) {
        match register {
            Register8::A => { self.a = value; }
            Register8::B => { self.b = value; }
            Register8::C => { self.c = value; }
            Register8::D => { self.d = value; }
            Register8::E => { self.e = value; }
            Register8::F => { self.f = value; }
            Register8::H => { self.h = value; }
            Register8::L => { self.l = value; }
            Register8::M => { bus.write_b(self.get_16(&Register16::HL), value); }
        }
    }
}

#[derive(Debug)]
pub struct Instruction8080
{
    pub length: u8,
    pub opcode: u8,
    pub action: InstructionAction,
    pub target: InstructionTarget
}

impl Instruction8080
{
    pub fn new(opcode: u8) -> Self {
        Self {
            length: 1, opcode,
            action: InstructionAction::None,
            target: InstructionTarget::None
        }
    }

    pub fn from_opcode(pc: u16, bus: &Box<dyn Bus8080>) -> Self {
        let opcode = bus.as_ref().read_b(pc);
        let (opcode_high, opcode_low) = ((opcode & 0xF0) >> 4, opcode & 0xF);
        let mut result = Instruction8080::new(opcode);

        // Shortcut tables for organization.
        const REGISTER8_TABLE_FIRST: [Register8; 4] = [Register8::B, Register8::D, Register8::H, Register8::M];
        const REGISTER8_TABLE_SECOND: [Register8; 4] = [Register8::C, Register8::E, Register8::L, Register8::A];
        const REGISTER8_TABLE_ALL: [Register8; 8] = [Register8::B, Register8::C, Register8::D, Register8::E, Register8::H, Register8::L, Register8::M, Register8::A];
        const REGISTER16_TABLE_FIRST: [Register16; 4] = [Register16::BC, Register16::DE, Register16::HL, Register16::SP];
        const REGISTER16_TABLE_SECOND: [Register16; 4] = [Register16::BC, Register16::DE, Register16::HL, Register16::DSW];
        const CONDITION_TABLE_FIRST: [Condition; 4] = [Condition::NotZero, Condition::NotCarry, Condition::PairtyOdd, Condition::Plus];
        const CONDITION_TABLE_SECOND: [Condition; 4] = [Condition::Zero, Condition::Carry, Condition::ParityEven, Condition::Minus];

        // Reference: https://pastraiser.com/cpu/i8080/i8080_opcodes.html
        match (opcode_high, opcode_low)
        {
        // First row start.

            // NOP
            (0x0..=0x3, 0x0) => { result.action = InstructionAction::Nothing; }

            // LXI
            (0x0..=0x3, 0x1) => {
                result.length += 2;
                result.action = InstructionAction::Load16 { register: REGISTER16_TABLE_FIRST[opcode_high as usize]};
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) }
            }
            
            // STAX
            (0x0..=0x1, 0x2) => {
                result.action = InstructionAction::StoreRegToMemory { register: Register8::A };
                result.target = InstructionTarget::Register16 { register: REGISTER16_TABLE_FIRST[opcode_high as usize] }                
            }

            // SHLD
            (0x2, 0x2) => {
                result.length += 2;
                result.action = InstructionAction::StoreReg16ToMemory { register: Register16::HL };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) }                 
            }

            // STA
            (0x3, 0x2) => {
                result.length += 2;
                result.action = InstructionAction::StoreRegToMemory { register: Register8::A };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) }
            }

            // INX
            (0x0..=0x3, 0x3) => { result.action = InstructionAction::Increment16 { register: REGISTER16_TABLE_FIRST[opcode_high as usize]}; }
    
            // INR first
            (0x0..=0x3, 0x4) => {
                result.action = InstructionAction::AddReg { register: REGISTER8_TABLE_FIRST[opcode_high as usize], carry: false };
                result.target = InstructionTarget::Immediate8 { value: 1 };
            }

            // DCR first
            (0x0..=0x3, 0x5) => {
                result.action = InstructionAction::SubReg { register: REGISTER8_TABLE_FIRST[opcode_high as usize], carry: false };
                result.target = InstructionTarget::Immediate8 { value: 1 };
            }

            // MVI first
            (0x0..=0x3, 0x6) => {
                result.length += 1;
                result.action = InstructionAction::MovReg { register: REGISTER8_TABLE_FIRST[opcode_high as usize] };
                result.target = InstructionTarget::Immediate8 { value: bus.read_b(pc + 1) }
            }

            // RLC / RAL
            (0x0..=0x1, 0x7) => { result.action = InstructionAction::RotateReg { register: Register8::A, right: false, arithmetic: opcode_high != 0 }; }

            // DAA
            (0x2, 0x7) => { result.action = InstructionAction::DAAReg { register: Register8::A }; }

            // STC
            (0x3, 0x7) => { result.action = InstructionAction::SetCarry { value: true } }

            // NOP
            (0x0..=0x3, 0x8) => { result.action = InstructionAction::Nothing; }

            // DAD
            (0x0..=0x3, 0x9) => {
                result.action = InstructionAction::Add16 { register: Register16::HL };
                result.target = InstructionTarget::Register16 { register: REGISTER16_TABLE_FIRST[opcode_high as usize] };
            }

            // LDAX
            (0x0..=0x1, 0xA) => {
                result.action = InstructionAction::LoadRegFromMemory { register: Register8::A };
                result.target = InstructionTarget::Register16 { register: REGISTER16_TABLE_FIRST[opcode_high as usize] };             
            }

            // LHLD
            (0x2, 0xA) => {
                result.length += 2;
                result.action = InstructionAction::LoadReg16FromMemory { register: Register16::HL };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };              
            }

            // LDA
            (0x3, 0xA) => {
                result.length += 2;
                result.action = InstructionAction::LoadRegFromMemory { register: Register8::A };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };
            }

            // DCX
            (0x0..=0x3, 0xB) => { result.action = InstructionAction::Decrement16 { register: REGISTER16_TABLE_FIRST[opcode_high as usize]}; }

            // INR second
            (0x0..=0x3, 0xC) => {
                result.action = InstructionAction::AddReg { register: REGISTER8_TABLE_SECOND[opcode_high as usize], carry: false };
                result.target = InstructionTarget::Immediate8 { value: 1 };
            }

            // DCR second
            (0x0..=0x3, 0xD) => {
                result.action = InstructionAction::SubReg { register: REGISTER8_TABLE_SECOND[opcode_high as usize], carry: false };
                result.target = InstructionTarget::Immediate8 { value: 1 };
            }

            // MVI second
            (0x0..=0x3, 0xE) => {
                result.length += 1;
                result.action = InstructionAction::MovReg { register: REGISTER8_TABLE_SECOND[opcode_high as usize] };
                result.target = InstructionTarget::Immediate8 { value: bus.read_b(pc + 1) }
            }

            // RRC / RAR
            (0x0..=0x1, 0xF) => { result.action = InstructionAction::RotateReg { register: Register8::A, right: true, arithmetic: opcode_high != 0 }; }

            // CMA
            (0x2, 0xF) => { result.action = InstructionAction::ComplementReg { register: Register8::A }; }

            // CMC
            (0x3, 0xF) => { result.action = InstructionAction::ComplementCarry; }

        // First row done.

        // Second row start.

            // MOV first
            (0x4..=0x7, 0x0..=0x7) => {
                result.action = InstructionAction::MovReg { register: REGISTER8_TABLE_FIRST[(opcode_high - 0x4) as usize] };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[opcode_low as usize] };
                
                // Mov M, M is actually Halt.
                if opcode == 0x76
                {
                    result.action = InstructionAction::Halt;
                    result.target = InstructionTarget::None;
                }
            }

            // MOV second
            (0x4..=0x7, 0x8..=0xF) => {
                result.action = InstructionAction::MovReg { register: REGISTER8_TABLE_SECOND[(opcode_high - 0x4) as usize] };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[(opcode_low - 0x8) as usize] };
            }

        // Second row done.

        // Third row start.

            // ADD / ADC
            (0x8, 0x0..=0xF) => {
                result.action = InstructionAction::AddReg { register: Register8::A,  carry: opcode_low >= 0x8 };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[(opcode_low % 0x8) as usize] };
            }
            
            // SUB / SBB
            (0x9, 0x0..=0xF) => {
                result.action = InstructionAction::SubReg { register: Register8::A,  carry: opcode_low >= 0x8 };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[(opcode_low % 0x8) as usize] };
            }
            
            // ANA
            (0xA, 0x0..=0x7) => {
                result.action = InstructionAction::AndReg { register: Register8::A };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[(opcode_low % 0x8) as usize] };
            }

            // XRA
            (0xA, 0x8..=0xF) => {
                result.action = InstructionAction::XorReg { register: Register8::A };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[(opcode_low % 0x8) as usize] };
            }

            // ORA
            (0xB, 0x0..=0x7) => {
                result.action = InstructionAction::OrReg { register: Register8::A };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[(opcode_low % 0x8) as usize] };
            }

            // CMP
            (0xB, 0x8..=0xF) => {
                result.action = InstructionAction::CompareReg { register: Register8::A };
                result.target = InstructionTarget::Register8 { register: REGISTER8_TABLE_ALL[(opcode_low % 0x8) as usize] };
            }

        // Third row done.

        // Fourth row start.

            // Conditional returns first
            (0xC..=0xF, 0x0) => {
                result.action = InstructionAction::Return { condition: CONDITION_TABLE_FIRST[(opcode_high - 0xC) as usize] };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };               
            }

            // Pop 16-bit
            (0xC..=0xF, 0x1) => { result.action = InstructionAction::Pop16 { register: REGISTER16_TABLE_SECOND[(opcode_high - 0xC) as usize] }; }

            // Conditional jumps first
            (0xC..=0xF, 0x2) => {
                result.length += 2;
                result.action = InstructionAction::Jump { condition: CONDITION_TABLE_FIRST[(opcode_high - 0xC) as usize] };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };               
            }

            // Unconditional jumps
            (0xC, 0x3) | (0xC, 0xB) => {
                result.length += 2;
                result.action = InstructionAction::Jump { condition: Condition::None };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };
            }

            // Out 8-bit
            (0xD, 0x3) => {
                result.length += 1;
                result.action = InstructionAction::Out8;
                result.target = InstructionTarget::Immediate8 { value: bus.read_b(pc + 1) }
            }

            // XTHL
            (0xE, 0x3) => { result.action = InstructionAction::ExchangeToStack; }

            // DI
            (0xF, 0x3) => { result.action = InstructionAction::SetInterrupts { enabled: false }; }

            // Conditional calls first
            (0xC..=0xF, 0x4) => {
                result.length += 2;
                result.action = InstructionAction::Call { condition: CONDITION_TABLE_FIRST[(opcode_high - 0xC) as usize] };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };               
            }

            // Push 16-bit
            (0xC..=0xF, 0x5) => {
                result.action = InstructionAction::Push16 { register: REGISTER16_TABLE_SECOND[(opcode_high - 0xC) as usize] };
            }

            // ADI
            (0xC, 0x6) => {
                result.length += 1;
                result.action = InstructionAction::AddReg { register: Register8::A, carry: false };
                result.target = InstructionTarget::Immediate8 { value:  bus.read_b(pc + 1) }
            }

            // SUI
            (0xD, 0x6) => {
                result.length += 1;
                result.action = InstructionAction::SubReg { register: Register8::A, carry: false };
                result.target = InstructionTarget::Immediate8 { value:  bus.read_b(pc + 1) }
            }

            // ANI
            (0xE, 0x6) => {
                result.length += 1;
                result.action = InstructionAction::AndReg { register: Register8::A };
                result.target = InstructionTarget::Immediate8 { value: bus.read_b(pc + 1) };
            }

            // ORI
            (0xF, 0x6) => {
                result.length += 1;
                result.action = InstructionAction::OrReg { register: Register8::A };
                result.target = InstructionTarget::Immediate8 { value:  bus.read_b(pc + 1) }
            }

            // Even resets
            (0xC..=0xF, 0x7) => {
                result.action = InstructionAction::Call { condition: Condition::None };
                result.target = InstructionTarget::Immediate16 { value: 0x0000 + 8 * ((opcode_high - 0xC) * 2) as u16 };                
            }

            // Conditional returns second
            (0xC..=0xF, 0x8) => {
                result.action = InstructionAction::Return { condition: CONDITION_TABLE_SECOND[(opcode_high - 0xC) as usize] };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };               
            }

            // Unconditional returns.
            (0xC..=0xD, 0x9) => {
                result.action = InstructionAction::Return { condition: Condition::None };
            }

            // PCHL
            (0xE, 0x9) => {
                result.action = InstructionAction::Call { condition: Condition::None };
                result.target = InstructionTarget::Register16 { register: Register16::HL };
            }

            // SPHL
            (0xF, 0x9) => {
                result.action = InstructionAction::Load16 { register: Register16::SP };
                result.target = InstructionTarget::Register16 { register: Register16::HL };
            }

            // Conditional jumps second
            (0xC..=0xF, 0xA) => {
                result.length += 2;
                result.action = InstructionAction::Jump { condition: CONDITION_TABLE_SECOND[(opcode_high - 0xC) as usize] };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };               
            }

            // IN
            (0xD, 0xB) => { result.action = InstructionAction::In8; }

            // Exchange
            (0xE, 0xB) => { result.action = InstructionAction::Exchange; }

            // EI
            (0xF, 0xB)  => { result.action = InstructionAction::SetInterrupts { enabled: true }; }

            // Conditional calls second.
            (0xC..=0xF, 0xC) => {
                result.length += 2;
                result.action = InstructionAction::Call { condition: CONDITION_TABLE_SECOND[(opcode_high - 0xC) as usize] };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };               
            }

            // Unconditional calls.
            (0xC..=0xF, 0xD) => {
                result.length += 2;
                result.action = InstructionAction::Call { condition: Condition::None };
                result.target = InstructionTarget::Immediate16 { value: bus.read_w(pc + 1) };
            }

            // ACI
            (0xC, 0xE) => {
                result.length += 1;
                result.action = InstructionAction::AddReg { register: Register8::A, carry: true };
                result.target = InstructionTarget::Immediate8 { value:  bus.read_b(pc + 1) }
            }

            // SBI
            (0xD, 0xE) => {
                result.length += 1;
                result.action = InstructionAction::SubReg { register: Register8::A, carry: true };
                result.target = InstructionTarget::Immediate8 { value:  bus.read_b(pc + 1) }
            }

            // XRI
            (0xE, 0xE) => {
                result.length += 1;
                result.action = InstructionAction::XorReg { register: Register8::A };
                result.target = InstructionTarget::Immediate8 { value: bus.read_b(pc + 1) };
            }

            // CPI
            (0xF, 0xE) => {
                result.length += 1;
                result.action = InstructionAction::CompareReg { register: Register8::A };
                result.target = InstructionTarget::Immediate8 { value:  bus.read_b(pc + 1) }
            }

            // Odd resets.
            (0xC..=0xF, 0xF) => {
                result.action = InstructionAction::Call { condition: Condition::None };
                result.target = InstructionTarget::Immediate16 { value: 0x0000 + 8 * ((opcode_high - 0xC) * 2 + 1) as u16 }; 
            }

        // Fourth row end.

            _ => { }        // Everything else does not exist.
        }
        result
    }
}