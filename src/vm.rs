use std::convert::From;
use std::error;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::*;

use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use bigdecimal::ToPrimitive;

use dcstack;
use instructions::*;
use parse;

#[derive(Debug)]
pub enum VMState {
    Continue,
    StackError(dcstack::DCError),
    InvalidInputRadix,
    InvalidOutputRadix,
    InvalidPrecision,
    InvalidCallStackOperation,
    NotImplemented,
    TerminatingReturn,
    TerminatingReturnEnclosing,
    NonTerminatingReturn(u64),
}

static CONTINUE: &'static str = "continue";
static INVALID_INPUT_RADIX: &'static str = "invalid input radix";
static INVALID_OUTPUT_RADIX: &'static str = "invalid output radix";
static INVALID_PRECISION: &'static str = "invalid precision";
static NOT_IMPLEMENTED: &'static str = "not implemented";
static TERMINATING_RETURN: &'static str = "terminating return";
static TERMINATING_RETURN_ENCLOSING: &'static str = "terminating return enclosing";
static NON_TERMINATING_RETURN: &'static str = "non terminating return";
static BAD_Q_NUMBER: &'static str = "Q command requires a number >= 1";

impl VMState {
    fn message(&self) -> &'static str {
        match self {
            &VMState::Continue => &CONTINUE,
            &VMState::InvalidInputRadix => &INVALID_INPUT_RADIX,
            &VMState::InvalidOutputRadix => &INVALID_OUTPUT_RADIX,
            &VMState::InvalidPrecision => &INVALID_PRECISION,
            &VMState::NotImplemented => &NOT_IMPLEMENTED,
            &VMState::InvalidCallStackOperation => &BAD_Q_NUMBER,
            &VMState::TerminatingReturn => &TERMINATING_RETURN,
            &VMState::TerminatingReturnEnclosing => &TERMINATING_RETURN_ENCLOSING,
            &VMState::NonTerminatingReturn(..) => &NON_TERMINATING_RETURN,
            &VMState::StackError(dcerror) => dcerror.message(),
        }
    }

    #[allow(dead_code)]
    fn stack_unwind(&self) -> u64 {
        match self {
            &VMState::NonTerminatingReturn(n) => n,
            &VMState::TerminatingReturnEnclosing => 1,
            &VMState::TerminatingReturn => 2,
            _other => 0,
        }
    }

    #[cfg(test)]
    fn is_ok(&self) -> bool {
        match self {
            &VMState::Continue
            | &VMState::NonTerminatingReturn(..)
            | &VMState::TerminatingReturnEnclosing
            | &VMState::TerminatingReturn => true,
            _s => false,
        }
    }

    #[cfg(test)]
    fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

impl From<dcstack::DCError> for VMState {
    fn from(error: dcstack::DCError) -> VMState {
        VMState::StackError(error)
    }
}

impl From<Result<(), dcstack::DCError>> for VMState {
    fn from(result: Result<(), dcstack::DCError>) -> VMState {
        match result {
            Ok(()) => VMState::Continue,
            Err(stack_error) => VMState::StackError(stack_error),
        }
    }
}

impl From<Result<VMState, dcstack::DCError>> for VMState {
    fn from(result: Result<VMState, dcstack::DCError>) -> VMState {
        match result {
            Ok(state) => state,
            Err(stack_error) => VMState::StackError(stack_error),
        }
    }
}

impl fmt::Display for VMState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message())?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum VMError {
    FmtError(fmt::Error),
    IoError(::std::io::Error),
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let type_ = match self {
            &VMError::FmtError(..) => "fmt error",
            &VMError::IoError(..) => "io error",
        };
        write!(f, "{} {}", type_, self.description())?;
        Ok(())
    }
}

impl error::Error for VMError {
    fn cause(&self) -> Option<&Error> {
        match self {
            &VMError::FmtError(ref error) => Some(error),
            &VMError::IoError(ref error) => Some(error),
        }
    }

    fn description(&self) -> &str {
        self.cause()
            .map(Error::description)
            .expect("there is always a cause")
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

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ReturnState {
    Done,
    TerminatingReturn,
    TerminatingReturnEnclosing,
    NonTerminatingReturn(u64),
}

impl ReturnState {
    fn next(&self) -> Self {
        match self {
            &ReturnState::Done => ReturnState::Done,
            &ReturnState::TerminatingReturn => ReturnState::TerminatingReturnEnclosing,
            &ReturnState::TerminatingReturnEnclosing => ReturnState::Done,
            &ReturnState::NonTerminatingReturn(0) => ReturnState::Done,
            &ReturnState::NonTerminatingReturn(n) => ReturnState::NonTerminatingReturn(n - 1),
        }
    }

    fn terminates_exec(&self) -> bool {
        match self {
            &ReturnState::Done => false,
            &ReturnState::TerminatingReturn => true,
            &ReturnState::TerminatingReturnEnclosing => true,
            &ReturnState::NonTerminatingReturn(_n) => true,
        }
    }
}

impl Default for ReturnState {
    fn default() -> Self {
        return ReturnState::Done;
    }
}

impl fmt::Display for ReturnState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(match self {
            &ReturnState::Done => f.write_str("Done"),
            &ReturnState::TerminatingReturn => f.write_str("TerminatingReturn"),
            &ReturnState::TerminatingReturnEnclosing => f.write_str("TerminatingReturnEnclosing"),
            &ReturnState::NonTerminatingReturn(n) => write!(f, "NonTerminatingReturn({})", n),
        }?)
    }
}

impl From<VMState> for ReturnState {
    fn from(vm_state: VMState) -> ReturnState {
        match vm_state {
            VMState::NonTerminatingReturn(n) => ReturnState::NonTerminatingReturn(n),
            VMState::TerminatingReturn => ReturnState::TerminatingReturn,
            VMState::TerminatingReturnEnclosing => ReturnState::TerminatingReturnEnclosing,
            _other => ReturnState::Done,
        }
    }
}

impl From<ReturnState> for VMState {
    fn from(return_state: ReturnState) -> VMState {
        match return_state {
            ReturnState::Done => VMState::Continue,
            ReturnState::NonTerminatingReturn(n) => VMState::NonTerminatingReturn(n),
            ReturnState::TerminatingReturn => VMState::TerminatingReturn,
            ReturnState::TerminatingReturnEnclosing => VMState::TerminatingReturnEnclosing,
        }
    }
}

#[derive(Debug)]
pub struct VM<W, WE>
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
    macro_level: u64,
}

macro_rules! bin_op {
    ($dcstack:expr; $e:expr) => {{
        if let Err(stack_error) = $dcstack.binary_apply_and_consume_tos($e) {
            VMState::StackError(stack_error)
        } else {
            VMState::Continue
        }
    }};
}

impl<W, WE> Default for VM<W, WE>
where
    W: Write + Default,
    WE: Write + Default,
{
    fn default() -> VM<W, WE> {
        VM {
            stack: dcstack::DCStack::new(),
            input_radix: 10,
            output_radix: 10,
            precision: 0,
            sink: W::default(),
            error_sink: WE::default(),
            macro_level: 0,
        }
    }
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
            macro_level: 0,
        }
    }

    pub fn sinks(self) -> (W, WE) {
        (self.sink, self.error_sink)
    }

    fn trace<T>(&self, where_: &str, i: &T) -> Result<(), io::Error>
    where
        T: fmt::Display,
    {
        if cfg!(feature = "tracevm") {
            let out = ::std::io::stderr();
            writeln!(
                out.lock(),
                "{} {} {} i:{} o:{} k:{} m:{}",
                where_,
                i,
                self.stack,
                self.input_radix,
                self.output_radix,
                self.precision,
                self.macro_level,
            )?;
            writeln!(out.lock(), "{:?}", ReturnState::NonTerminatingReturn(1))?;
        }
        Ok(())
    }

    fn eval(&mut self, program: &Program) -> Result<ReturnState, io::Error> {
        for instruction in &program.instructions {
            self.trace("__enter_eval__", instruction)?;
            match self.eval_instruction(&instruction) {
                Err(VMError::IoError(ioerror)) => {
                    self.trace("__exit_eval__ ioerror", instruction)?;
                    return Err(ioerror);
                }
                Err(error) => {
                    self.trace("__exit_eval__ error", instruction)?;
                    writeln!(self.error_sink, "dc: {}", error)?;
                }
                Ok(vm_state) => {
                    self.trace("__exit_eval__", instruction)?;
                    match vm_state {
                        VMState::Continue => continue,
                        r @ VMState::TerminatingReturn
                        | r @ VMState::TerminatingReturnEnclosing
                        | r @ VMState::NonTerminatingReturn(..) => return Ok(ReturnState::from(r)),
                        error => {
                            // we do not really need to go out here
                            writeln!(self.error_sink, "dc: {}", error)?;
                        }
                    }
                }
            };
        }
        Ok(ReturnState::Done)
    }

    pub fn execute(&mut self, program_text: &[u8]) -> Result<ReturnState, io::Error> {
        let res = match parse::parse(program_text) {
            Ok(program) => {
                self.trace("__enter_execute__", &program)?;
                self.eval(&program)?
            }
            Err(parse_error) => {
                self.trace("__enter_execute__ parse error", &parse_error.program)?;
                let eval_res = self.eval(&parse_error.program)?;
                if eval_res.terminates_exec() {
                    eval_res
                } else {
                    writeln!(self.error_sink, "dc: {}", parse_error)?;
                    self.execute(parse_error.unparsed)?
                }
            }
        };
        self.trace("__exit_execute__", &UTF8Adapter { program_text })?;
        Ok(res)
    }

    fn execute_macro(&mut self, program_text: &[u8]) -> Result<ReturnState, io::Error> {
        self.macro_level += 1; 
        match self.execute(program_text) {
            Ok(result_status) => {
                self.macro_level -= 1; 
                Ok(result_status.next())
            }
            Err(error) => {
                self.macro_level -= 1; 
                Err(error)
            },
        }
    }

    #[inline]
    fn precision_i64(&self) -> i64 {
        self.precision as i64
    }

    fn eval_instruction(&mut self, instruction: &Instruction) -> Result<VMState, VMError> {
        let state = match instruction {
            &Instruction::Nop => VMState::Continue,
            &Instruction::Num(integer, fraction) => {
                if let Err(stack_error) =
                    self.stack
                        .push_bytes_as_num(integer, fraction, self.input_radix)
                {
                    VMState::StackError(stack_error)
                } else {
                    VMState::Continue
                }
            }
            &Instruction::Str(text) => {
                self.stack.push_str(text);
                VMState::Continue
            }
            // print
            &Instruction::PrintLN => match self.stack.peek() {
                Ok(tos) => {
                    writeln!(self.sink, "{}", tos.to_str_radix(self.output_radix))?;
                    VMState::Continue
                }
                Err(stack_error) => VMState::StackError(stack_error),
            },
            &Instruction::PrintPop => match self.stack.pop() {
                Ok(tos) => {
                    writeln!(self.sink, "{}", tos.to_str_radix(self.output_radix))?;
                    VMState::Continue
                }
                Err(stack_error) => VMState::StackError(stack_error),
            },
            &Instruction::PrettyPrint => VMState::NotImplemented,
            &Instruction::PrintStack => {
                self.stack.write_to(&mut self.sink, self.output_radix)?;
                VMState::Continue
            }
            // arithmetic
            &Instruction::Add => bin_op![self.stack; BigDecimal::add_assign],
            &Instruction::Sub => bin_op![self.stack; BigDecimal::sub_assign],
            &Instruction::Mul => bin_op![self.stack; BigDecimal::mul_assign],
            &Instruction::Div => {
                let precision = self.precision_i64();
                bin_op![self.stack; |dest, other| {*dest = &*dest / other; *dest = dest.with_scale(precision)}]
            }
            &Instruction::Mod => bin_op![self.stack; |dest, other| *dest = &*dest % other],
            &Instruction::Divmod => VMState::NotImplemented,
            &Instruction::Exp => VMState::NotImplemented,
            &Instruction::Modexp => VMState::NotImplemented,
            &Instruction::Sqrt => {
                // TODO: this implementation is buggy as it goes through fp
                let precision = self.precision_i64();
                VMState::from(self.stack.apply_tos_num_opt(|n| {
                    ToPrimitive::to_f64(n)
                        .map(f64::sqrt)
                        .and_then(BigDecimal::from_f64)
                        .map(|n| n.with_scale(precision))
                }))
            }
            // stack
            &Instruction::Clear => VMState::from(self.stack.clear()),
            &Instruction::Dup => VMState::from(self.stack.dup()),
            &Instruction::Swap => VMState::from(self.stack.swap()),
            // register
            &Instruction::RegisterOperation { .. } => VMState::NotImplemented,
            // parameters
            &Instruction::SetInputRadix => {
                VMState::from(self.stack.pop_num().map(|n| self.set_input_radix(n)))
            }
            &Instruction::GetInputRadix => {
                self.stack.push_num(self.input_radix);
                VMState::Continue
            }
            &Instruction::SetOutputRadix => {
                VMState::from(self.stack.pop_num().map(|n| self.set_output_radix(n)))
            }
            &Instruction::GetOutputRadix => {
                self.stack.push_num(self.output_radix);
                VMState::Continue
            }
            &Instruction::SetPrecision => {
                VMState::from(self.stack.pop_num().map(|n| self.set_precision(n)))
            }
            &Instruction::GetPrecision => {
                self.stack.push_num(self.precision);
                VMState::Continue
            }
            // string
            &Instruction::OpToString => VMState::NotImplemented,
            &Instruction::ExecuteTos => match self.stack.pop_str() {
                Ok(bytes) => self.execute_macro(&bytes)?.into(),
                Err(stack_error) => VMState::StackError(stack_error),
            },
            &Instruction::ExecuteInput => VMState::NotImplemented,
            &Instruction::ReturnCaller => VMState::TerminatingReturn,
            &Instruction::ReturnN => match self.stack.pop_num() {
                Ok(levels) => {
                    if let Some(levels) = levels.to_u64() {
                        VMState::NonTerminatingReturn(levels)
                    } else {
                        VMState::InvalidCallStackOperation
                    }
                }
                Err(stack_error) => VMState::StackError(stack_error),
            },
            // status enquiry
            &Instruction::Digits => VMState::NotImplemented,
            &Instruction::FractionDigits => VMState::NotImplemented,
            &Instruction::StackDepth => {
                let len = self.stack.len();
                self.stack.push_num(len as u64);
                VMState::Continue
            }
            // miscellaneous
            &Instruction::System(..) => VMState::NotImplemented,
            &Instruction::Comment(..) => VMState::Continue,
        };
        Ok(state)
    }

    fn set_input_radix(&mut self, radix: BigDecimal) -> VMState {
        let (n, scale) = radix.as_bigint_and_exponent();
        if scale != 0 {
            return VMState::InvalidInputRadix;
        }

        match n.to_u32() {
            Some(n) => {
                if n < 2 || n > 16 {
                    VMState::InvalidInputRadix
                } else {
                    self.input_radix = n;
                    VMState::Continue
                }
            }
            None => VMState::InvalidInputRadix,
        }
    }

    fn set_output_radix(&mut self, radix: BigDecimal) -> VMState {
        let (n, scale) = radix.as_bigint_and_exponent();
        if scale != 0 {
            return VMState::InvalidOutputRadix;
        }

        match n.to_u32() {
            Some(n) => {
                if n < 2 || n > 16 {
                    VMState::InvalidOutputRadix
                } else {
                    self.output_radix = n;
                    VMState::Continue
                }
            }
            None => VMState::InvalidOutputRadix,
        }
    }

    fn set_precision(&mut self, precision: BigDecimal) -> VMState {
        if let Some(value) = precision.to_u64() {
            if value >= 2 {
                self.precision = value;
                return VMState::Continue;
            }
        }
        VMState::InvalidPrecision
    }
}

struct UTF8Adapter<'a> {
    program_text: &'a [u8],
}

impl<'a> fmt::Display for UTF8Adapter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use std::fmt::Write;
        for ch in self.program_text {
            f.write_char(*ch as char)?
        }
        Ok(())
    }
}

#[cfg(test)]
macro_rules! test_vm_num {
    ($ ( $ x : expr ) , * ) => (
        InMemoryVM {
            stack: dcstack_num![ $( $x ),* ],
            .. InMemoryVM::default()
        }
    );
}

#[cfg(test)]
macro_rules! test_vm {
    ($ ( $ x : expr ) , * ) => (
        InMemoryVM {
            stack: dcstack![ $( $x ),* ],
            .. InMemoryVM::default()
        }
    );
}

#[cfg(test)]
type InMemoryVM = VM<Vec<u8>, Vec<u8>>;

#[test]
fn test_input_radix() {
    let mut vm = InMemoryVM::default();
    assert!(vm.set_input_radix(BigDecimal::from(10)).is_ok());
}

#[test]
fn test_input_radix_fail() {
    let mut vm = InMemoryVM::default();
    assert!(vm.set_input_radix(BigDecimal::from(50)).is_err());
}

#[test]
fn test_output_radix() {
    let mut vm = InMemoryVM::default();
    assert!(vm.set_output_radix(BigDecimal::from(10)).is_ok());
}

#[test]
fn test_output_radix_fail() {
    let mut vm = InMemoryVM::default();
    assert!(vm.set_output_radix(BigDecimal::from(1)).is_err());
}

#[test]
fn test_precision() {
    let mut vm = InMemoryVM::default();
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
        Ok(VMState::StackError(dcstack::DCError::StackEmpty)) => {}
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
        Ok(VMState::StackError(dcstack::DCError::NonNumericValue)) => {}
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
        Ok(VMState::StackError(dcstack::DCError::NonNumericValue)) => {}
        _ => assert!(false),
    }
    assert!(vm.stack.len() == 2);
}

#[test]
fn test_exec() {
    let mut vm = VM::default();
    assert!(vm.execute(b"110.0[p]x").is_ok());
    let (actual_output, actual_error) = vm.sinks();

    assert_eq!(
        ("110.0\n".to_string(), String::new()),
        (
            String::from_utf8(actual_output).expect("utf8 output"),
            String::from_utf8(actual_error).expect("utf8 error")
        ),
    );
}

macro_rules! test_exec {
    ($name:ident; $program:expr; $expected_output:expr) => {
        #[test]
        fn $name() {
            let program = $program;
            let mut vm = VM::default();
            match parse::parse(program) {
                Err(error) => {
                    println!("parse error {:?}", error);
                    assert!(false);
                }
                Ok(program) => println!("{}", program),
            }

            assert!(vm.execute(program).is_ok());
            let (actual_output, actual_error) = vm.sinks();

            assert_eq!(
                (String::from($expected_output), String::new()),
                (
                    String::from_utf8(actual_output).expect("utf8 output"),
                    String::from_utf8(actual_error).expect("utf8 error")
                ),
            );
        }
    };
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

test_exec![_10p; b"10p";"10\n"];
test_exec![add; b"10 20 + p";"30\n"];
test_exec![sub; b"10 20 - p";"-10\n"];
test_exec![mul; b"10 20 * p";"200\n"];
test_exec![div; b"10 20 / p";"0\n"];
test_exec![mod_; b"10 20 % p";"10\n"];

test_exec![swap;b"10 20 rf";"10\n20\n"];

test_exec![clear;b"10cf";""];
test_exec![clear_empty;b"cf";""];

test_exec![quit_now;b"q10p"; ""];
test_exec![quit_now1;b"q"; ""];
test_exec![quit;b"[qp]x10p"; ""];
test_exec![quit2;b"[qp]x"; ""];
test_exec![no_quit_macro_depth;b"[qp][x]x10p";"10\n"];
