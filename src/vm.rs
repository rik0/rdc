use std::io;
use std::fmt;
use std::error;
use std::error::Error;
use std::convert::From;
use std::ops::*;
use std::io::prelude::*;

use bigdecimal::BigDecimal;

use num::ToPrimitive;

use dcstack;
use instructions::*;

#[derive(Debug)]
pub enum VMError {
    StackError(dcstack::DCError),
    FmtError(fmt::Error),
    IoError(::std::io::Error),
    InvalidInputRadix,
    InvalidOutputRadix,
    InvalidPrecision,
}

static INVALID_INPUT_RADIX: &'static str = "invalid input radix";
static INVALID_OUTPUT_RADIX: &'static str = "invalid output radix";
static INVALID_PRECISION: &'static str = "invalid precision";

impl VMError {
    fn message(&self) -> &str {
        match self {
            &VMError::InvalidInputRadix => &INVALID_INPUT_RADIX,
            &VMError::InvalidOutputRadix => &INVALID_OUTPUT_RADIX,
            &VMError::InvalidPrecision => &INVALID_PRECISION,
            &VMError::StackError(dcerror) => dcerror.message(),
            // TODO ugly but it works
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

pub struct VM<'a> {
    stack: dcstack::DCStack,
    input_radix: u32,  // [2,16]
    output_radix: u32, // >= 2
    precision: u64,    // > 0, always in decimal
    sink: &'a mut Write,
}

impl<'a> VM<'a> {
    pub fn new(w: &mut Write) -> VM {
        VM {
            stack: dcstack::DCStack::new(),
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: w,
        }
    }

    pub fn eval(&mut self, instructions: &[Instruction]) -> Result<(), VMError> {
        for instruction in instructions {
            self.eval_instruction(instruction)?;
        }
        Ok(())
    }

    fn print(&mut self, element: dcstack::MemoryCell) -> Result<(), VMError> {
        match element {
            dcstack::MemoryCell::Num(n) => {
                let (bigint, exp) = n.into_bigint_and_exponent();
                let s = bigint.to_str_radix(self.output_radix);
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
            &Instruction::PrintLN => {
                let tos = self.stack.clone_tos()?;
                self.print(tos)
            }
            &Instruction::PrintPop => {
                let tos = self.stack.pop()?;
                self.print(tos)
            }
            &Instruction::Add => {
                self.stack.apply_and_consume_tos(move |mut dest, tos| {
                    dest += tos;
                    dest
                })?;
                Ok(())
                // TODO might be a bug, stack should not be popped
                // might be easier to make prechecks...
            }
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
            _ => Ok(()),
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

#[test]
fn test_input_radix() {
    let mut buffer = Vec::new();
    let mut vm = VM::new(&mut buffer);
    assert!(vm.set_input_radix(BigDecimal::from(10)).is_ok());
}

#[test]
fn test_input_radix_fail() {
    let mut buffer = Vec::new();
    let mut vm = VM::new(&mut buffer);
    assert!(vm.set_input_radix(BigDecimal::from(50)).is_err());
}

#[test]
fn test_output_radix() {
    let mut buffer = Vec::new();
    let mut vm = VM::new(&mut buffer);
    assert!(vm.set_output_radix(BigDecimal::from(10)).is_ok());
}

#[test]
fn test_output_radix_fail() {
    let mut buffer = Vec::new();
    let mut vm = VM::new(&mut buffer);
    assert!(vm.set_output_radix(BigDecimal::from(50)).is_err());
}

#[test]
fn test_precision() {
    let mut buffer = Vec::new();
    let mut vm = VM::new(&mut buffer);
    assert!(vm.set_precision(BigDecimal::from(10)).is_ok());
}
