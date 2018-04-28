
use std::fmt;
use std::error;
use std::convert::{From, Into};

use num;
use dcstack;
use instructions::*;


#[derive(Clone, Debug, PartialEq)]
pub enum VMError {
    StackError(dcstack::DCError),
    InvalidInputRadix,
    InvalidOutputRadix,
}

static INVALID_INPUT_RADIX: &'static str = "invalid input radix";
static INVALID_OUTPUT_RADIX: &'static str = "invalid input radix";

impl VMError {
    fn message(&self) -> &'static str {
        match self {
            &VMError::InvalidInputRadix => &INVALID_INPUT_RADIX,
            &VMError::InvalidOutputRadix => &INVALID_OUTPUT_RADIX,
            &VMError::StackError(dcerror) => dcerror.message(),
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

pub struct VM<'a, T: num::Num + From<u32> > {
    stack: dcstack::DCStack<'a, T>,
    input_radix: u8,  // [2,16]
    output_radix: u8, // >= 2
    precision: u64,   // > 0, always in decimal
}

impl<'a, T: num::Num + From<u32>> VM<'a, T> 
    where u64: From<T>
{
    pub fn new() -> VM<'a, T> {
        VM::<T> {
            stack: dcstack::DCStack::new(),
            input_radix: 10,
            output_radix: 10,
            precision: 0,
        }
    }

    pub fn eval(&mut self, instructions: &[Instruction]) -> Result<(), VMError> {
        for instruction in instructions {
            self.eval_instruction(instruction)?;
        }
        Ok(())
    }

    fn eval_instruction(&mut self, instruction: &Instruction) -> Result<(), VMError> {
        match instruction {
            &Instruction:: SetInputRadix => {
                let n : T = self.stack.pop_num()?;
                self.set_input_radix(n)
            }
            _ => Ok(())
        }
    }

    fn set_input_radix(&mut self, radix: T) -> Result<(), VMError> {
        if radix != T::from(10) {
            return Err(VMError::InvalidInputRadix);
        }
        self.input_radix = 10;
        return Ok(());
    }

    fn set_output_radix(&mut self, radix: T) -> Result<(), VMError> {
        if radix != T::from(10) {
            return Err(VMError::InvalidOutputRadix);
        }
        self.output_radix = 10;
        return Ok(());
    }

    fn set_precision(&mut self, precision: T) -> Result<(), VMError> {
        self.precision = T::into(precision);
        return Ok(());
    }
}

#[test]
fn test_input_radix() {
    let mut vm = VM::<u64>::new();
    assert!(vm.set_input_radix(10).is_ok());
}

#[test]
fn test_input_radix_fail() {
    let mut vm = VM::<u64>::new();
    assert!(vm.set_input_radix(50).is_err());
}

#[test]
fn test_output_radix() {
    let mut vm = VM::<u64>::new();
    assert!(vm.set_output_radix(10).is_ok());
}

#[test]
fn test_output_radix_fail() {
    let mut vm = VM::<u64>::new();
    assert!(vm.set_output_radix(50).is_err());
}

#[test]
fn test_precision() {
    let mut vm = VM::<u64>::new();
    assert!(vm.set_precision(10).is_ok());
}