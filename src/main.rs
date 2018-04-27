use std::io::Write;
use std::str;
use std::str::FromStr;
use std::error;
use std::error::Error;
use std::f64;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

type Register = u8;
//type ProgramText = &[u8];

// todo remove T (we must always keep the chars... what about using references to the main program for all types?)
#[derive(Copy, Clone, Debug, PartialEq)]
enum RegisterOperationType {
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
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Instruction<'a> {
    Nop,
    Num(&'a [u8]),
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
    MakeString,
    OpToString,
    ExecuteInput,
    ReturnCaller,
    ReturnN,
    // status enquiry
    Digits,
    FractionDigits,
    StackDepth,
    // miscellaneous
    System(&'a [u8]),
    Comment,
    SetArray,
    GetArray,
}

#[derive(Clone, Debug, PartialEq)]
enum ParserErrorType {
    IllegalState(String),
    InvalidCharacter(u8),
    NumParseError(usize, usize, String),
    EOP(String),
}

#[derive(Clone, Debug, PartialEq)]
enum ParserState {
    TopLevel,
    Error(usize, ParserErrorType),
    Num {
        start: usize,
        end: usize,
        seen_dot: bool,
    },
    Command {
        start: usize,
        end: usize,
    },
    Register(RegisterOperationType),
    Mark,
    End,
}

#[derive(Clone, Debug, PartialEq)]
struct ParserError {
    position: usize,
    error_type: ParserErrorType,
}

fn parse(program_text: &[u8]) -> Result<Vec<Instruction>, ParserError> {
    let mut state = ParserState::TopLevel;
    let mut instructions = Vec::new();
    let mut position: usize = 0;

    if program_text.len() == 0 {
        return Ok(instructions);
    }
    loop {
        match state {
            ParserState::End => break,
            ParserState::Error(_, _) => break,
            ParserState::TopLevel => {
                if position >= program_text.len() {
                    break;
                }
                let ch = program_text[position];

                match ch {
                    0 => instructions.push(Instruction::Nop),
                    b'.' => {
                        // here we effectively consume one character, so we must go through the increment
                        state = ParserState::Num {
                            start: position,
                            end: position + 1,
                            seen_dot: true,
                        };
                    }
                    b'0'...b'9' => {
                        // here we effectively consume one character, so we must go through the increment
                        state = ParserState::Num {
                            start: position,
                            end: position + 1,
                            seen_dot: false,
                        };
                    }
                    b'p' => instructions.push(Instruction::PrintLN),
                    b'n' => instructions.push(Instruction::PrintPop),
                    b'P' => instructions.push(Instruction::PrettyPrint),
                    b'f' => instructions.push(Instruction::PrintStack),
                    b'+' => instructions.push(Instruction::Add),
                    b'-' => instructions.push(Instruction::Sub),
                    b'*' => instructions.push(Instruction::Mul),
                    b'/' => instructions.push(Instruction::Div),
                    b'%' => instructions.push(Instruction::Mod),
                    b'~' => instructions.push(Instruction::Divmod),
                    b'^' => instructions.push(Instruction::Exp),
                    b'|' => instructions.push(Instruction::Modexp),
                    b'v' => instructions.push(Instruction::Sqrt),
                    b'c' => instructions.push(Instruction::Clear),
                    b'd' => instructions.push(Instruction::Dup),
                    b'r' => instructions.push(Instruction::Swap),
                    b's' => state = ParserState::Register(RegisterOperationType::Store),
                    b'l' => state = ParserState::Register(RegisterOperationType::Load),
                    b'S' => state = ParserState::Register(RegisterOperationType::StoreStack),
                    b'L' => state = ParserState::Register(RegisterOperationType::LoadStack),
                    b'>' => state = ParserState::Register(RegisterOperationType::TosGtExecute),
                    b'<' => state = ParserState::Register(RegisterOperationType::TosLtExecute),
                    b'=' => state = ParserState::Register(RegisterOperationType::TosEqExecute),
                    b'!' => state = ParserState::Mark,
                    b'i' => instructions.push(Instruction::SetInputRadix),
                    b'o' => instructions.push(Instruction::SetOutputRadix),
                    b'k' => instructions.push(Instruction::SetPrecision),
                    b'I' => instructions.push(Instruction::GetInputRadix),
                    b'O' => instructions.push(Instruction::GetOutputRadix),
                    b'K' => instructions.push(Instruction::GetPrecision),
                    b' ' | b'\n' => (), // do nothing
                    ch => {
                        state = ParserState::Error(position, ParserErrorType::InvalidCharacter(ch));
                        continue;
                    }
                }

                position += 1;
            }
            ParserState::Num {
                start,
                end,
                seen_dot,
            } => {
                if position >= program_text.len() {
                    instructions.push(Instruction::Num(&program_text[start..end]));
                    state = ParserState::End;
                    break;
                }
                let ch = program_text[position];
                match (seen_dot, ch) {
                    (false, b'.') => {
                        // if we are here, we were alredy building a number and finally we got the .
                        state = ParserState::Num {
                            start,
                            end: end + 1,
                            seen_dot: true,
                        };
                        position += 1;
                    }
                    (_, b'0'...b'9') => {
                        state = ParserState::Num {
                            start,
                            end: end + 1,
                            seen_dot: seen_dot,
                        };
                        position += 1;
                    }
                    (true, b'.') | _ => {
                        // it means it initiates a new number, store the old and start again
                        // we must not advance the position: note that this means we are looping
                        // a bit more than necessary, but it makes the logic simpler
                        instructions.push(Instruction::Num(&program_text[start..end]));
                        state = ParserState::TopLevel;
                    }
                }
            }
            ParserState::Register(register_operation_type) => {
                if position >= program_text.len() {
                    state = ParserState::Error(
                        position,
                        ParserErrorType::EOP("was expecting a register".to_string()),
                    );
                    break;
                }
                let ch = program_text[position];
                instructions.push(Instruction::RegisterOperation(register_operation_type, ch));
                state = ParserState::TopLevel;
                position += 1;
            }
            ParserState::Mark => {
                if position >= program_text.len() {
                    break;
                }
                let ch = program_text[position];

                match ch {
                    b'>' => {
                        state = ParserState::Register(RegisterOperationType::TosGeExecute);
                    }
                    b'<' => {
                        state = ParserState::Register(RegisterOperationType::TosLeExecute);
                    }
                    b'=' => {
                        state = ParserState::Register(RegisterOperationType::TosNeExecute);
                    }
                    _ => {
                        state = ParserState::Command {
                            start: position,
                            end: position + 1,
                        };
                    }
                }
                position += 1;
            }
            ParserState::Command { start, end } => {
                if position >= program_text.len() {
                    instructions.push(Instruction::System(&program_text[start..end]));
                    state = ParserState::End;
                    break;
                }
                let ch = program_text[position];
                if ch == b'\n' {
                    instructions.push(Instruction::System(&program_text[start..end]));
                    state = ParserState::TopLevel;
                } else {
                    state = ParserState::Command {
                        start,
                        end: end+1,
                    }
                }
                position += 1; // let us skip the newline or mark the char as visited
            }
        }
    }

    return match state {
        ParserState::Error(position, error_type) => Err(ParserError {
            position,
            error_type,
        }),
        ParserState::End => Ok(instructions),
        //ParserState::TopLevel => Err(ParserError{position: position, error_type: ParserErrorType::IllegalState("parsing stopped".to_string())}),
        ParserState::TopLevel => Ok(instructions),
        _other => Err(ParserError {
            position: position,
            error_type: ParserErrorType::IllegalState("not sure".to_string()),
        }),
    };
}

// fn ascii_to_num<T>(bytes: &[u8]) -> Result<T, String>
//     where T: FromStr + Default,
//           <T as str::FromStr>::Err: error::Error
// {
//     return match str::from_utf8(bytes) {
//         Ok(".") => Ok(T::default()),
//         Ok(chars) => {
//             match T::from_str(chars) {
//                 Ok(n) => Ok(n),
//                 Err(error) => Err(error.description().to_string()),
//             }
//         }
//         Err(utf8error) => Err(utf8error.description().to_string())
//     }
// }

macro_rules! parse_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (input, expected) = $value;
            assert_eq!(expected, parse(input.as_bytes()));
        }
    )*
    }
}

parse_tests! {
    parse_test_empty: ("", Ok(vec![])),
    parse_test_invalid: ("z", Err(ParserError{position: 0, error_type: ParserErrorType::InvalidCharacter(b'z')})),
    parse_test_zero: ("\0", Ok(vec![Instruction::Nop])),
    parse_test_p: ("p", Ok(vec![Instruction::PrintLN])),
    parse_test_n: ("n", Ok(vec![Instruction::PrintPop])),
    parse_test_p2: ("P", Ok(vec![Instruction::PrettyPrint])),
    parse_test_f: ("f", Ok(vec![Instruction::PrintStack])),
    parse_test_add: ("+", Ok(vec![Instruction::Add])),
    parse_test_sub: ("-", Ok(vec![Instruction::Sub])),
    parse_test_mul: ("*", Ok(vec![Instruction::Mul])),
    parse_test_div: ("/", Ok(vec![Instruction::Div])),
    parse_test_mod: ("%", Ok(vec![Instruction::Mod])),
    parse_test_divmod: ("~", Ok(vec![Instruction::Divmod])),
    parse_test_exp: ("^", Ok(vec![Instruction::Exp])),
    parse_test_expmod: ("|", Ok(vec![Instruction::Modexp])),
    parse_test_v: ("v", Ok(vec![Instruction::Sqrt])),
    parse_test_c: ("c", Ok(vec![Instruction::Clear])),
    parse_test_d: ("d", Ok(vec![Instruction::Dup])),
    parse_test_r: ("r", Ok(vec![Instruction::Swap])),
    parse_test_i: ("i", Ok(vec![Instruction::SetInputRadix])),
    parse_test_o: ("o", Ok(vec![Instruction::SetOutputRadix])),
    parse_test_k: ("k", Ok(vec![Instruction::SetPrecision])),
    parse_test_i2: ("I", Ok(vec![Instruction::GetInputRadix])),
    parse_test_o2: ("O", Ok(vec![Instruction::GetOutputRadix])),
    parse_test_k2: ("K", Ok(vec![Instruction::GetPrecision])),
    parse_test_0: ("0", Ok(vec![Instruction::Num("0".as_bytes())])),
    parse_test_0dot: ("0.", Ok(vec![Instruction::Num("0.".as_bytes())])),
    parse_test_dot0: (".0", Ok(vec![Instruction::Num(".0".as_bytes())])),
    parse_test_132763: ("132763", Ok(vec![Instruction::Num("132763".as_bytes())])),
    parse_test_1: ("1", Ok(vec![Instruction::Num("1".as_bytes())])),
    parse_test_1dot: ("1.", Ok(vec![Instruction::Num("1.".as_bytes())])),
    parse_test_dot1: (".1", Ok(vec![Instruction::Num(".1".as_bytes())])),
    parse_test_dot: (".", Ok(vec![Instruction::Num(".".as_bytes())])),
    parse_test_dotdot: ("..", Ok(vec![Instruction::Num(".".as_bytes()), Instruction::Num(".".as_bytes())])),
    parse_test_dot_dot: (". .", Ok(vec![Instruction::Num(".".as_bytes()), Instruction::Num(".".as_bytes())])),
    parse_test_zero_dot_zero: ("0.0", Ok(vec![Instruction::Num("0.0".as_bytes())])),
    parse_test_00: ("00", Ok(vec![Instruction::Num("00".as_bytes())])),
    parse_test_11: ("11", Ok(vec![Instruction::Num("11".as_bytes())])),
    parse_test_0_0: ("0 0", Ok(vec![Instruction::Num("0".as_bytes()), Instruction::Num("0".as_bytes())])),
    parse_test_1_1: ("1 1", Ok(vec![Instruction::Num("1".as_bytes()), Instruction::Num("1".as_bytes())])),
    parse_test_la: ("la", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::Load, b'a' as Register)])),
    parse_test_sa: ("sa", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::Store, b'a' as Register)])),
    parse_test_l2a: ("La", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::LoadStack, b'a' as Register)])),
    parse_test_s2a: ("Sa", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::StoreStack, b'a' as Register)])),
    parse_test_lta: ("<a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosLtExecute, b'a' as Register)])),
    parse_test_gta: (">a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosGtExecute, b'a' as Register)])),
    parse_test_eqa: ("=a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosEqExecute, b'a' as Register)])),
    parse_test_lea: ("!<a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosLeExecute, b'a' as Register)])),
    parse_test_gea: ("!>a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosGeExecute, b'a' as Register)])),
    parse_test_nea: ("!=a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosNeExecute, b'a' as Register)])),
    parse_test_sysa: ("!a", Ok(vec![Instruction::System("a".as_bytes())])),
    parse_test_ltagt: ("<>", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosLtExecute, b'>' as Register)])),
}

#[test]
fn testparse() {
    let (input, expected) = ("", Ok(vec![]));
    assert_eq!(expected, parse(input.as_bytes()));
}

#[derive(Debug, Clone)]
struct VMError {
    message: String,
}

struct VM {
    input_radix: u8,  // [2,16]
    output_radix: u8, // >= 2
    precision: u64,   // > 0, always in decimal
}

impl VM {
    fn new() -> VM {
        VM {
            input_radix: 10,
            output_radix: 10,
            precision: 0,
        }
    }

    fn eval(&mut self, _instructions: &[Instruction]) {}

    fn set_input_radix(&mut self, radix: u8) -> Result<(), VMError> {
        if radix != 10 {
            return Err(VMError {
                message: "invalid radix".to_string(),
            });
        }
        self.input_radix = radix;
        return Ok(());
    }

    fn set_output_radix(&mut self, radix: u8) -> Result<(), VMError> {
        if radix != 10 {
            return Err(VMError {
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
    let mut vm = VM::new();
    assert!(vm.set_input_radix(10).is_ok());
}

#[test]
fn test_input_radix_fail() {
    let mut vm = VM::new();
    assert!(vm.set_input_radix(50).is_err());
}

#[test]
fn test_output_radix() {
    let mut vm = VM::new();
    assert!(vm.set_output_radix(10).is_ok());
}

#[test]
fn test_output_radix_fail() {
    let mut vm = VM::new();
    assert!(vm.set_output_radix(50).is_err());
}

#[test]
fn test_precision() {
    let mut vm = VM::new();
    assert!(vm.set_precision(10).is_ok());
}

enum ProgramSource {
    Text(String),
    File(String),
}

impl ProgramSource {
    fn into_bytes<'a>(self, buffer: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        match self {
            ProgramSource::Text(text_str) => {
                return Ok(buffer.write(text_str.as_bytes())?);
            }
            ProgramSource::File(filename) => {
                let path = Path::new(&filename);
                let mut file = File::open(path)?;
                return Ok(file.read_to_end(buffer)?);
            }
        }
    }
}

fn main() {
    // let us implement the real app to understand approaches to ownership

    let mut vm = VM::new();

    let mut args = std::env::args().skip(1);

    let mut program_sources = Vec::new();
    let mut positional_program_sources = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-e" | "--expression" => match args.next() {
                Some(text) => program_sources.push(ProgramSource::Text(text)),
                None => print_help(1),
            },
            "-f" | "--file" => match args.next() {
                Some(file) => program_sources.push(ProgramSource::File(file)),
                None => print_help(1),
            },
            "-h" | "--help" => print_help(0),
            "-v" | "--version" => print_version(0),
            _ => {
                positional_program_sources.push(ProgramSource::File(arg.to_string()));
            }
        }
    }

    for program_source in program_sources {
        let mut source_code = Vec::new();
        match program_source.into_bytes(&mut source_code) {
            Ok(bytes) => match parse(&source_code[..bytes]) {
                Err(parse_error) => {
                    // TODO I should use a description
                    let _ = writeln!(
                        std::io::stderr(),
                        "{:?} at col {}",
                        parse_error.error_type,
                        parse_error.position
                    ).unwrap();
                }
                Ok(instructions) => {
                    vm.eval(&instructions[..]);
                }
            },
            Err(error) => {
                eprintln!("error processing file {}", error);
            }
        }
    }
}

fn print_help(code: i32) {
    std::process::exit(code);
}

fn print_version(code: i32) {
    std::process::exit(code);
}
