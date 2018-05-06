use std::fmt;
use std::io;
use std::error;
use std::error::Error;
use std::convert::From;
use std::ops::*;
use std::io::prelude::*;

use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use bigdecimal::FromPrimitive;

use parse;
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

pub struct VM<W: Write, WE: Write>
where
    W: Write,
    WE: Write,
{
    stack: dcstack::DCStack,
    input_radix: u32,  // [2,16]
    output_radix: u32, // >= 2
    precision: u64,    // > 0, always in decimal
    sink: W,
    error_sink: WE,
}

macro_rules! bin_op {
    ($dcstack:expr; $e: expr) => ({
       Ok($dcstack.binary_apply_and_consume_tos($e)?)
    });
}

impl<W, WE> VM<W, WE>
where
    W: Write,
    WE: Write,
{
    pub fn new(w: W, esink: WE) -> VM<W, WE> {
        VM {
            stack: dcstack::DCStack::new(),
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: w,
            error_sink: esink,
        }
    }

    pub fn sinks(self) -> (W, WE) {
        (self.sink, self.error_sink)
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

    pub fn execute(&mut self, program_text: &[u8]) -> Result<(), io::Error> {
        match parse::parse(program_text) {
            Ok(instructions) => Ok(self.eval(&instructions)?),
            Err(parse_error) => {
                self.eval(&parse_error.instructions)?;
                writeln!(self.error_sink, "dc: {}", parse_error)?;
                self.execute(parse_error.unparsed)
            }
        }
    }

    fn eval_instruction(&mut self, instruction: &Instruction) -> Result<(), VMError> {
        match instruction {
            &Instruction::Nop => Ok(()),
            &Instruction::Num(integer, fraction) => {
                self.stack
                    .push_bytes_as_num(integer, fraction, self.input_radix)?;
                Ok(())
            }
            &Instruction::Str(text) => {
                self.stack.push_str(text);
                Ok(())
            }
            // print
            &Instruction::PrintLN => {
                let tos = self.stack.peek()?;
                Ok(writeln!(
                    self.sink,
                    "{}",
                    tos.to_str_radix(self.output_radix)
                )?)
            }
            &Instruction::PrintPop => {
                let tos = self.stack.pop()?;
                Ok(writeln!(
                    self.sink,
                    "{}",
                    tos.to_str_radix(self.output_radix)
                )?)
            }
            &Instruction::PrettyPrint => Err(VMError::NotImplemented),
            &Instruction::PrintStack => Ok(self.stack.write_to(&mut self.sink, self.output_radix)?),
            // arithmetic
            &Instruction::Add => bin_op![self.stack; BigDecimal::add_assign],
            &Instruction::Sub => bin_op![self.stack; BigDecimal::sub_assign],
            &Instruction::Mul => bin_op![self.stack; BigDecimal::mul_assign],
            &Instruction::Div => bin_op![self.stack; |dest, other| *dest = &*dest / other],
            &Instruction::Mod => bin_op![self.stack; |dest, other| *dest = &*dest % other],
            &Instruction::Divmod => Err(VMError::NotImplemented),
            &Instruction::Exp => Err(VMError::NotImplemented),
            &Instruction::Modexp => Err(VMError::NotImplemented),
            &Instruction::Sqrt => {
                // TODO: this implementation is buggy as it goes through fp
                let precision = self.precision as i64;
                Ok(self.stack.apply_tos_num_opt(|n| {
                    ToPrimitive::to_f64(n)
                        .map(f64::sqrt)
                        .and_then(BigDecimal::from_f64)
                        .map(|n| n.with_scale(precision))
                })?)
            }
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
                Ok(self.execute(&bytes)?)
            }
            &Instruction::ExecuteInput => Err(VMError::NotImplemented),
            &Instruction::ReturnN => Err(VMError::NotImplemented),
            &Instruction::ReturnCaller => Err(VMError::NotImplemented),
            // status enquiry
            &Instruction::Digits => Err(VMError::NotImplemented),
            &Instruction::FractionDigits => Err(VMError::NotImplemented),
            &Instruction::StackDepth => {
                let len = self.stack.len();
                self.stack.push_num(len as u64);
                Ok(())
            }
            // miscellaneous
            &Instruction::System(..) => Err(VMError::NotImplemented),
            &Instruction::Comment(..) => Ok(()),
        }
    }

    fn set_input_radix(&mut self, radix: BigDecimal) -> Result<(), VMError> {
        let (n, scale) = radix.as_bigint_and_exponent();
        if scale != 0 {
            return Err(VMError::InvalidInputRadix);
        }

        n.to_u32()
            .and_then(|n| {
                if n < 2 || n > 16 {
                    None
                } else {
                    self.input_radix = n;
                    Some(())
                }
            })
            .ok_or(VMError::InvalidInputRadix)
    }

    fn set_output_radix(&mut self, radix: BigDecimal) -> Result<(), VMError> {
        let (n, scale) = radix.as_bigint_and_exponent();
        if scale != 0 {
            return Err(VMError::InvalidOutputRadix);
        }

        n.to_u32()
            .and_then(|n| {
                if n < 2 {
                    None
                } else {
                    self.output_radix = n;
                    Some(())
                }
            })
            .ok_or(VMError::InvalidOutputRadix)
    }

    fn set_precision(&mut self, precision: BigDecimal) -> Result<(), VMError> {
        if let Some(value) = precision.to_u64() {
            if value >= 2 {
                self.precision = value;
                return Ok(());
            }
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
    assert!(vm.set_output_radix(BigDecimal::from(1)).is_err());
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

#[test]
fn test_exec() {
    let mut output: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    {
        let mut vm = VM {
            stack: dcstack::DCStack::new(),
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: &mut output,
            error_sink: &mut err,
        };
        assert!(vm.execute(b"110.0[p]x").is_ok())
    }

    assert_eq!(Vec::from("110.0\n"), output);
}

macro_rules! test_exec {
    ($name:ident; $program:expr; $expected_output:expr) => (
        #[test]
        fn $name() {
            let mut output: Vec<u8> = Vec::new();
            let mut error: Vec<u8> = Vec::new();
            {
                let mut vm = VM {
                    stack: dcstack::DCStack::new(),
                    input_radix: 10,
                    output_radix: 10,
                    precision: 0,
                    sink: &mut output,
                    error_sink: &mut error,
                };
                println!("{:?}", parse::parse($program));
                assert!(vm.execute($program).is_ok())
            }

            assert_eq!((Vec::from($expected_output), String::new()), (output, String::from_utf8(error).unwrap()));
        }
    )
}

test_exec![test_num;b"10";""];
test_exec![test_p;b"10p";"10\n"];
test_exec![test_p2;b"10n";"10\n"];
test_exec![test_p2p;b"10nzp";"10\n0\n"];

test_exec![test_oct;b"8o 8p";"10\n"];

// this does not fail because it does not parse non decimal
test_exec![test_input_set_get_base;b"8iIp";"8\n"];
#[cfg(feature = "parse_all_bases")]
test_exec![test_input_hex;b"16iAp";"10\n"];
#[cfg(feature = "parse_all_bases")]
test_exec![test_input_hex_aa;b"16iAAp";"170\n"];
#[cfg(feature = "parse_all_bases")]
test_exec![test_input_hex_dec;b"16iA.Ap";"10.6\n"];
#[cfg(feature = "parse_all_bases")]
test_exec![test_input_bin_dec;b"2i1.101p";"1.625\n"];
// test_exec![test_input_oct;b"8i 10p";"8\n"];

// sqrt
// test_exec![test_sqrt;b".4vp";"0.6\n"];
// test_exec![test_sqrt_with_precision;b"2p.4vp";"0.63\n"];
test_exec![test_sqrt_on_int;b"4vp";"2\n"];
test_exec![test_sqrt_on_int_with_precision;b"2k4vp";"2.00\n"];
test_exec![test_sqrt_on_int_irr_resul;b"2vp";"1\n"];
test_exec![test_sqrt_on_int_with_precision_irr_resul;b"2k2vp";"1.41\n"];
