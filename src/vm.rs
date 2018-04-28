
use std::fmt;
use std::error;

use num;
use dcstack;
use instructions::*;


#[derive(Clone, Debug, PartialEq)]
enum VMErrorType {
    InvalidInputRadix,
    InvalidOutputRadix,
}

static INVALID_INPUT_RADIX: &'static str = "invalid input radix";
static INVALID_OUTPUT_RADIX: &'static str = "invalid input radix";

impl fmt::Display for VMErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let message = match self {
            &VMErrorType::InvalidInputRadix => &INVALID_INPUT_RADIX,
            &VMErrorType::InvalidOutputRadix => &INVALID_OUTPUT_RADIX,
        };
        write!(f, "{}", message)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct VMError {
    error_type: VMErrorType,
    message: String,
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl error::Error for VMError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub struct VM<'a, T: num::Num> {
    stack: dcstack::DCStack<'a, T>,
    input_radix: u8,  // [2,16]
    output_radix: u8, // >= 2
    precision: u64,   // > 0, always in decimal
}

impl<'a, T: num::Num+Clone> VM<'a, T> {
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

            }
            _ => {}
        }
        Ok(())
    }

    fn set_input_radix(&mut self, radix: u8) -> Result<(), VMError> {
        if radix != 10 {
            return Err(VMError {
                error_type: VMErrorType::InvalidInputRadix,
                message: "invalid radix".to_string(),
            });
        }
        self.input_radix = radix;
        return Ok(());
    }

    fn set_output_radix(&mut self, radix: u8) -> Result<(), VMError> {
        if radix != 10 {
            return Err(VMError {
                error_type: VMErrorType::InvalidOutputRadix,
                message: "invalid radix".to_string(),
            });
        }
        self.output_radix = radix;
        return Ok(());
    }

    fn set_precision(&mut self, precision: u64) -> Result<(), VMError> {
        self.precision = precision;
        return Ok(());
    }
}

#[test]
fn test_input_radix() {
    let mut vm = VM::<f64>::new();
    assert!(vm.set_input_radix(10).is_ok());
}

#[test]
fn test_input_radix_fail() {
    let mut vm = VM::<f64>::new();
    assert!(vm.set_input_radix(50).is_err());
}

#[test]
fn test_output_radix() {
    let mut vm = VM::<f64>::new();
    assert!(vm.set_output_radix(10).is_ok());
}

#[test]
fn test_output_radix_fail() {
    let mut vm = VM::<f64>::new();
    assert!(vm.set_output_radix(50).is_err());
}

#[test]
fn test_precision() {
    let mut vm = VM::<f64>::new();
    assert!(vm.set_precision(10).is_ok());
}