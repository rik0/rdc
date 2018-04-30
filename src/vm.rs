use std::fmt;
use std::io;
use std::error;
use std::error::Error;
use std::convert::From;
use std::ops::*;
use std::io::prelude::*;

use bigdecimal::BigDecimal;

use num::ToPrimitive;

use parse;
use dcstack;
use instructions::*;

#[derive(Debug)]
pub enum VMError {
    StackError(dcstack::DCError),
    FmtError(fmt::Error),
    IoError(::std::io::Error),
    ParseError(parse::ParserError),
    InvalidInputRadix,
    InvalidOutputRadix,
    InvalidPrecision,
    NotImplemented,
}

static INVALID_INPUT_RADIX: &'static str = "invalid input radix";
static INVALID_OUTPUT_RADIX: &'static str = "invalid output radix";
static INVALID_PRECISION: &'static str = "invalid precision";
static NOT_IMPLEMENTED: &'static str = "not implemented";

impl VMError {
    fn message(&self) -> &str {
        match self {
            &VMError::InvalidInputRadix => &INVALID_INPUT_RADIX,
            &VMError::InvalidOutputRadix => &INVALID_OUTPUT_RADIX,
            &VMError::InvalidPrecision => &INVALID_PRECISION,
            &VMError::NotImplemented => &NOT_IMPLEMENTED,
            &VMError::StackError(dcerror) => dcerror.message(),
            &VMError::ParseError(ref parse_error) => &Box::new(parse_error.description()),
            // TODO ugly but it works: display should only be a simple message and we should
            // do the interesting stuff in fmt, which can allocate memory more easily
            &VMError::FmtError(ref fmterror) => &Box::new(fmterror.description()),
            &VMError::IoError(ref ioerror) => &Box::new(ioerror.description()),
        }
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message())?;
        Ok(())
    }
}

impl error::Error for VMError {
    fn description(&self) -> &str {
        self.message()
    }
}

impl From<dcstack::DCError> for VMError {
    fn from(error: dcstack::DCError) -> VMError {
        VMError::StackError(error)
    }
}

impl From<fmt::Error> for VMError {
    fn from(error: fmt::Error) -> VMError {
        VMError::FmtError(error)
    }
}

impl From<::std::io::Error> for VMError {
    fn from(error: ::std::io::Error) -> VMError {
        VMError::IoError(error)
    }
}

impl From<parse::ParserError> for VMError {
    fn from(error: parse::ParserError) -> VMError {
        VMError::ParseError(error)
    }
}

pub struct VM<'a, 'b> {
    stack: dcstack::DCStack,
    input_radix: u32,  // [2,16]
    output_radix: u32, // >= 2
    precision: u64,    // > 0, always in decimal
    sink: &'a mut Write,
    error_sink: &'b mut Write,
}

macro_rules! bin_op {
    ($dcstack:expr; $e: expr) => ({
       Ok($dcstack.binary_apply_and_consume_tos($e)?)
    });
}

impl<'a, 'b> VM<'a, 'b> {
    pub fn new(w: &'a mut Write, esink: &'b mut Write) -> VM<'a, 'b> {
        VM {
            stack: dcstack::DCStack::new(),
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: w,
            error_sink: esink,
        }
    }

    pub fn eval(&mut self, instructions: &[Instruction]) -> Result<(), io::Error> {
        for instruction in instructions {
            match self.eval_instruction(instruction) {
                Err(VMError::IoError(ioerror)) => return Err(ioerror),
                Err(error) => writeln!(self.error_sink, "dc: {}", error)?,
                Ok(..) => {}
            }
        }
        Ok(())
    }

    pub fn execute(&mut self, program_text: &[u8]) -> Result<(), VMError> {
        let instructions = parse::parse(program_text)?;
        Ok(self.eval(&instructions)?)
    }

    fn print(&mut self, element: dcstack::MemoryCell) -> Result<(), VMError> {
        match element {
            dcstack::MemoryCell::Num(n) => {
                let (bigint, _exp) = n.into_bigint_and_exponent();
                let s = bigint.to_str_radix(self.output_radix);
                // TODO ignoring exp means we are not printing the .
                writeln!(self.sink, "{}", s)?;
            }
            dcstack::MemoryCell::Str(s) => {
                writeln!(self.sink, "{}", String::from_utf8_lossy(&s))?;
            }
        }
        Ok(())
    }

    fn eval_instruction(&mut self, instruction: &Instruction) -> Result<(), VMError> {
        match instruction {
            &Instruction::Nop => Ok(()),
            &Instruction::Num(text) => {
                self.stack.push_bytes_as_num(text, self.input_radix)?;
                Ok(())
            }
            &Instruction::Str(text) => {
                self.stack.push_str(text);
                Ok(())
            }
            // print
            &Instruction::PrintLN => {
                let tos = self.stack.clone_tos()?;
                self.print(tos)
            }
            &Instruction::PrintPop => {
                let tos = self.stack.pop()?;
                self.print(tos)
            }
            &Instruction::PrettyPrint => Err(VMError::NotImplemented),
            &Instruction::PrintStack => Err(VMError::NotImplemented),
            // arithmetic
            &Instruction::Add => bin_op![self.stack; BigDecimal::add_assign],
            &Instruction::Sub => bin_op![self.stack; BigDecimal::sub_assign],
            &Instruction::Mul => bin_op![self.stack; BigDecimal::mul_assign],
            &Instruction::Div => Err(VMError::NotImplemented),
            &Instruction::Mod => Err(VMError::NotImplemented),
            &Instruction::Divmod => Err(VMError::NotImplemented),
            &Instruction::Exp => Err(VMError::NotImplemented),
            &Instruction::Modexp => Err(VMError::NotImplemented),
            &Instruction::Sqrt => Err(VMError::NotImplemented),
            // stack
            &Instruction::Clear => Err(VMError::NotImplemented),
            &Instruction::Dup => Err(VMError::NotImplemented),
            &Instruction::Swap => Err(VMError::NotImplemented),
            // register
            &Instruction::RegisterOperation { .. } => Err(VMError::NotImplemented),
            // parameters
            &Instruction::SetInputRadix => {
                let n: BigDecimal = self.stack.pop_num()?;
                self.set_input_radix(n)
            }
            &Instruction::GetInputRadix => {
                self.stack.push_num(self.input_radix);
                Ok(())
            }
            &Instruction::SetOutputRadix => {
                let n: BigDecimal = self.stack.pop_num()?;
                self.set_output_radix(n)
            }
            &Instruction::GetOutputRadix => {
                self.stack.push_num(self.output_radix);
                Ok(())
            }
            &Instruction::SetPrecision => {
                let n: BigDecimal = self.stack.pop_num()?;
                self.set_precision(n)
            }
            &Instruction::GetPrecision => {
                self.stack.push_num(self.precision);
                Ok(())
            }
            // string
            &Instruction::OpToString => Err(VMError::NotImplemented),
            &Instruction::ExecuteTos => {
                let bytes = self.stack.pop_str()?;
                self.execute(&bytes)
            }
            &Instruction::ExecuteInput => Err(VMError::NotImplemented),
            &Instruction::ReturnN => Err(VMError::NotImplemented),
            &Instruction::ReturnCaller => Err(VMError::NotImplemented),
            // status enquiry
            &Instruction::Digits => Err(VMError::NotImplemented),
            &Instruction::FractionDigits => Err(VMError::NotImplemented),
            &Instruction::StackDepth => Err(VMError::NotImplemented),
            // miscellaneous
            &Instruction::System(..) => Err(VMError::NotImplemented),
            &Instruction::Comment(..) => Ok(()),
        }
    }

    fn set_input_radix(&mut self, radix: BigDecimal) -> Result<(), VMError> {
        if radix != BigDecimal::from(10) {
            return Err(VMError::InvalidInputRadix);
        }
        self.input_radix = 10;
        return Ok(());
    }

    fn set_output_radix(&mut self, radix: BigDecimal) -> Result<(), VMError> {
        if radix != BigDecimal::from(10) {
            return Err(VMError::InvalidOutputRadix);
        }
        self.output_radix = 10;
        return Ok(());
    }

    fn set_precision(&mut self, precision: BigDecimal) -> Result<(), VMError> {
        if let Some(value) = precision.to_u64() {
            self.precision = value;
            return Ok(());
        }
        Err(VMError::InvalidPrecision)
    }
}

#[cfg(test)]
macro_rules! empty_test_vm {
    () => (
        VM {
            stack: dcstack::DCStack::new(),
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: &mut Vec::new(),
            error_sink: &mut Vec::new(),
        }
    );
}

#[cfg(test)]
macro_rules! test_vm_num {
    ($ ( $ x : expr ) , * ) => (
        VM {
            stack: dcstack_num![ $( $x ),* ],
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: &mut Vec::new(),
            error_sink: &mut Vec::new(),
        }
    );
}

#[cfg(test)]
macro_rules! test_vm {
    ($ ( $ x : expr ) , * ) => (
        VM {
            stack: dcstack![ $( $x ),* ],
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: &mut Vec::new(),
            error_sink: &mut Vec::new(),
        }
    );
}

#[test]
fn test_input_radix() {
    let mut vm = empty_test_vm!();
    assert!(vm.set_input_radix(BigDecimal::from(10)).is_ok());
}

#[test]
fn test_input_radix_fail() {
    let mut vm = empty_test_vm!();
    assert!(vm.set_input_radix(BigDecimal::from(50)).is_err());
}

#[test]
fn test_output_radix() {
    let mut vm = empty_test_vm!();
    assert!(vm.set_output_radix(BigDecimal::from(10)).is_ok());
}

#[test]
fn test_output_radix_fail() {
    let mut vm = empty_test_vm!();
    assert!(vm.set_output_radix(BigDecimal::from(50)).is_err());
}

#[test]
fn test_precision() {
    let mut vm = empty_test_vm!();
    assert!(vm.set_precision(BigDecimal::from(10)).is_ok());
}

#[test]
fn test_add_happy() {
    let mut vm = test_vm_num!(4, 5);
    let res = vm.eval_instruction(&Instruction::Add);
    assert!(res.is_ok());
    assert_eq!(
        BigDecimal::from(9),
        vm.stack.pop_num().expect("should have been a number")
    )
}

#[test]
fn test_add_empty() {
    let mut vm = test_vm_num!();
    let res = vm.eval_instruction(&Instruction::Add);
    match res {
        Err(VMError::StackError(dcstack::DCError::StackEmpty)) => {}
        _ => assert!(false),
    }
}

#[test]
fn test_add_tos_not_num() {
    let mut vm = test_vm!(
        dcstack::MemoryCell::from(32),
        dcstack::MemoryCell::from_string("no")
    );
    let res = vm.eval_instruction(&Instruction::Add);
    match res {
        Err(VMError::StackError(dcstack::DCError::NonNumericValue)) => {}
        _ => assert!(false),
    }
    assert!(vm.stack.len() == 2);
}

#[test]
fn test_add_other_not_num() {
    let mut vm = test_vm!(
        dcstack::MemoryCell::from_string("no"),
        dcstack::MemoryCell::from(32)
    );
    let res = vm.eval_instruction(&Instruction::Add);
    match res {
        Err(VMError::StackError(dcstack::DCError::NonNumericValue)) => {}
        _ => assert!(false),
    }
    assert!(vm.stack.len() == 2);
}
