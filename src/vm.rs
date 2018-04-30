use std::fmt;
use std::io;
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
            &Instruction::PrintLN => {
                let tos = self.stack.clone_tos()?;
                self.print(tos)
            }
            &Instruction::PrintPop => {
                let tos = self.stack.pop()?;
                self.print(tos)
            }
            &Instruction::Add => bin_op![self.stack; BigDecimal::add_assign],
            &Instruction::Sub => bin_op![self.stack; BigDecimal::sub_assign],
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
