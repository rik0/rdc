use std::fmt;

pub type Register = u8;
//type ProgramText = &[u8];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RegisterOperationType {
    Store,
    Load,
    StoreStack,
    LoadStack,
    TosGeExecute,
    TosGtExecute,
    TosLeExecute,
    TosLtExecute,
    TosEqExecute,
    TosNeExecute,
    SetArray,
    GetArray,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction<'a> {
    Nop,
    Num(&'a [u8], &'a [u8]),
    Str(&'a [u8]),
    // print
    PrintLN,
    PrintPop,
    PrettyPrint,
    PrintStack,
    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Divmod,
    Exp,
    Modexp,
    Sqrt,
    // stack
    Clear,
    Dup,
    Swap,
    // registers
    RegisterOperation(RegisterOperationType, Register),
    // parameters
    SetInputRadix,
    SetOutputRadix,
    SetPrecision,
    GetInputRadix,
    GetOutputRadix,
    GetPrecision,
    // string
    OpToString,
    ExecuteTos,
    ExecuteInput,
    ReturnCaller,
    ReturnN,
    // status enquiry
    Digits,
    FractionDigits,
    StackDepth,
    // miscellaneous
    System(&'a [u8]),
    Comment(&'a [u8]),
}

fn allocate_str(v: &[u8]) -> String {
    String::from_utf8(Vec::from(v)).expect("allocate_str utf8 error")
}

impl<'a> fmt::Display for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Instruction::Nop => f.write_str("\0"),
            &Instruction::Num(ref integer, ref fractional) => {
                if fractional.len() > 0 {
                    write!(f, "{}.{}", allocate_str(integer), allocate_str(fractional))
                }  else {
                    write!(f, "{}", allocate_str(integer))
                }
            }
            &Instruction::Str(ref text) => write!(f, "[{}]", allocate_str(text)),
            &Instruction::PrintLN => f.write_str("p"),
            &Instruction::PrintPop => f.write_str("n"),
            &Instruction::PrettyPrint => f.write_str("P"),
            &Instruction::PrintStack => f.write_str("f"),
            &Instruction::Add => f.write_str("+"),
            &Instruction::Sub => f.write_str("-"),
            &Instruction::Mul => f.write_str("*"),
            &Instruction::Div => f.write_str("/"),
            &Instruction::Mod => f.write_str("%"),
            &Instruction::Divmod => f.write_str("~"),
            &Instruction::Exp => f.write_str("^"),
            &Instruction::Modexp => f.write_str("|"),
            &Instruction::Sqrt => f.write_str("v"),
            &Instruction::Clear => f.write_str("c"),
            &Instruction::Dup => f.write_str("d"),
            &Instruction::Swap => f.write_str("r"),
            &Instruction::RegisterOperation(optype, reg) => {
                match optype {
                    RegisterOperationType::Store => f.write_str("s")?,
                    RegisterOperationType::Load => f.write_str("l")?,
                    RegisterOperationType::StoreStack => f.write_str("S")?,
                    RegisterOperationType::LoadStack => f.write_str("L")?,
                    RegisterOperationType::TosGeExecute => f.write_str("!<")?,
                    RegisterOperationType::TosGtExecute => f.write_str(">")?,
                    RegisterOperationType::TosLeExecute => f.write_str("!>")?,
                    RegisterOperationType::TosLtExecute => f.write_str("<")?,
                    RegisterOperationType::TosEqExecute => f.write_str("=")?,
                    RegisterOperationType::TosNeExecute => f.write_str("!=")?,
                    RegisterOperationType::SetArray => f.write_str(":")?,
                    RegisterOperationType::GetArray => f.write_str(";")?,
                }
                f.write_fmt(format_args!["{}", reg as char])
            }
            &Instruction::SetInputRadix => f.write_str("i"),
            &Instruction::SetOutputRadix => f.write_str("o"),
            &Instruction::SetPrecision => f.write_str("k"),
            &Instruction::GetInputRadix => f.write_str("I"),
            &Instruction::GetOutputRadix => f.write_str("O"),
            &Instruction::GetPrecision => f.write_str("K"),
            &Instruction::OpToString => f.write_str("a"),
            &Instruction::ExecuteTos => f.write_str("x"),
            &Instruction::ExecuteInput => f.write_str("?"),
            &Instruction::ReturnCaller => f.write_str("q"),
            &Instruction::ReturnN => f.write_str("Q"),
            &Instruction::Digits => f.write_str("Z"),
            &Instruction::FractionDigits => f.write_str("X"),
            &Instruction::StackDepth => f.write_str("z"),
            &Instruction::System(ref command) => {
                write!(f, "!{}\n", allocate_str(command))
            }
            &Instruction::Comment(ref comment) => {
                write!(f, "#{}\n", allocate_str(comment))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Program<'a> {

    pub instructions: Vec<Instruction<'a>>
}

impl <'a> Program<'a> {
    pub fn push(&mut self, instruction: Instruction<'a>) {
        self.instructions.push(instruction)
    }
}

impl <'a> fmt::Display for Program<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            use std::fmt::Write;
            f.write_char('`')?;
            for instruction in &self.instructions {
                instruction.fmt(f)?
            }
            f.write_char('`')?;
            Ok(())
        }
}
