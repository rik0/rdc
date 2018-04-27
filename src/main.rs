use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Range;

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
    //Str(&'a [u8]),
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
    //MakeString,
    //OpToString,
    //ExecuteInput,
    //ReturnCaller,
    //ReturnN,
    // status enquiry
    // Digits,
    // FractionDigits,
    // StackDepth,
    // miscellaneous
    System(&'a [u8]),
    //Comment,
    //SetArray,
    //GetArray,
}

#[derive(Clone, Debug, PartialEq)]
enum ParserErrorType {
    InvalidCharacter(u8),
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
    ReadUntilByte{terminator: u8, range: Range<usize>},
    Command {
        start: usize,
        end: usize,
    },
    Register(RegisterOperationType),
    Mark,
}

#[derive(Clone, Debug, PartialEq)]
struct ParserError {
    position: usize,
    error_type: ParserErrorType,
}

macro_rules! incrementing {
    ($identifier:ident; $case:block) => ({
        let result : ParserState = $case;
        $identifier += 1;
        result
    });
    ($identifier:ident, $amount:expr; $case:block) => ({
        let result : ParserState = $case;
        $identifier += expr;
        result
    });
    ($identifier:ident; $next_state:expr) => ({
        let result : ParserState = $next_state;
        $identifier += 1;
        result
    });
    ($identifier:ident, $amount:expr; $case:expr) => ({
        let result : ParserState = $expr;
        $identifier += expr;
        result
    });
}

macro_rules! push_and_next_state {
    ($instructions:ident; $instruction:expr; $next_state:expr ) => ({
        $instructions.push($instruction);
        $next_state
    });
}

macro_rules! push_and_toplevel {
    ($instructions:ident; $instruction:expr) => (
        push_and_next_state![$instructions; $instruction; ParserState::TopLevel]
    ); 
}


fn parse(program_text: &[u8]) -> Result<Vec<Instruction>, ParserError> {
    let mut state = ParserState::TopLevel;
    let mut instructions = Vec::new();
    let mut position: usize = 0;

    loop {
        if position >= program_text.len() {
            return match state {
                ParserState::Error(position, error_type) => Err(ParserError {
                    position,
                    error_type,
                }),
                ParserState::TopLevel => Ok(instructions),
                ParserState::Num { start, end, seen_dot: _} => {
                    instructions.push(Instruction::Num(&program_text[start..end]));
                    Ok(instructions)
                }
                ParserState::Register(_) => Err(ParserError{
                        position,
                        error_type: ParserErrorType::EOP("was expecting a register".to_string()),
                    }),
                ParserState::Mark => Ok(instructions),
                ParserState::Command { start, end } => {
                    instructions.push(Instruction::System(&program_text[start..end]));
                    Ok(instructions)
                }
                ParserState::ReadUntilByte{terminator: _terminator , range } => {
                    instructions.push(Instruction::System(&program_text[range]));
                    Ok(instructions)
                }
            }
        }
        state = match (state, program_text[position]) {
            (ParserState::Error(position, error_type), _) => return Err(ParserError{position, error_type}),
            (ParserState::TopLevel, 0) => incrementing![position; push_and_toplevel![instructions; Instruction::Nop]],
            (ParserState::TopLevel, b'.') =>  incrementing![position; ParserState::Num {
                    start: position,
                    end: position + 1,
                    seen_dot: true,
            }],
            (ParserState::TopLevel, b'0'...b'9') => incrementing![position; ParserState::Num {
                    start: position,
                    end: position + 1,
                    seen_dot: false,
            }],
            (ParserState::TopLevel, b'p') => incrementing![position; push_and_toplevel![instructions; Instruction::PrintLN]],
            (ParserState::TopLevel, b'n') => incrementing![position; push_and_toplevel![instructions; Instruction::PrintPop]],
            (ParserState::TopLevel, b'P') => incrementing![position; push_and_toplevel![instructions; Instruction::PrettyPrint]],
            (ParserState::TopLevel, b'f') => incrementing![position; push_and_toplevel![instructions; Instruction::PrintStack]],
            (ParserState::TopLevel, b'+') => incrementing![position; push_and_toplevel![instructions; Instruction::Add]],
            (ParserState::TopLevel, b'-') => incrementing![position; push_and_toplevel![instructions; Instruction::Sub]],
            (ParserState::TopLevel, b'*') => incrementing![position; push_and_toplevel![instructions; Instruction::Mul]],
            (ParserState::TopLevel, b'/') => incrementing![position; push_and_toplevel![instructions; Instruction::Div]],
            (ParserState::TopLevel, b'%') => incrementing![position; push_and_toplevel![instructions; Instruction::Mod]],
            (ParserState::TopLevel, b'~') => incrementing![position; push_and_toplevel![instructions; Instruction::Divmod]],
            (ParserState::TopLevel, b'^') => incrementing![position; push_and_toplevel![instructions; Instruction::Exp]],
            (ParserState::TopLevel, b'|') => incrementing![position; push_and_toplevel![instructions; Instruction::Modexp]],
            (ParserState::TopLevel, b'v') => incrementing![position; push_and_toplevel![instructions; Instruction::Sqrt]],
            (ParserState::TopLevel, b'c') => incrementing![position; push_and_toplevel![instructions; Instruction::Clear]],
            (ParserState::TopLevel, b'd') => incrementing![position;push_and_toplevel![instructions; Instruction::Dup]],
            (ParserState::TopLevel, b'r') => incrementing![position;push_and_toplevel![instructions; Instruction::Swap]],
            (ParserState::TopLevel, b's') => incrementing![position;ParserState::Register(RegisterOperationType::Store)],
            (ParserState::TopLevel, b'l') => incrementing![position;ParserState::Register(RegisterOperationType::Load)],
            (ParserState::TopLevel, b'S') => incrementing![position;ParserState::Register(RegisterOperationType::StoreStack)],
            (ParserState::TopLevel, b'L') => incrementing![position;ParserState::Register(RegisterOperationType::LoadStack)],
            (ParserState::TopLevel, b'>') => incrementing![position;ParserState::Register(RegisterOperationType::TosGtExecute)],
            (ParserState::TopLevel, b'<') => incrementing![position;ParserState::Register(RegisterOperationType::TosLtExecute)],
            (ParserState::TopLevel, b'=') => incrementing![position;ParserState::Register(RegisterOperationType::TosEqExecute)],
            (ParserState::TopLevel, b'!') => incrementing![position;ParserState::Mark],
            (ParserState::TopLevel, b'i') => incrementing![position;push_and_toplevel![instructions; Instruction::SetInputRadix]],
            (ParserState::TopLevel, b'o') => incrementing![position;push_and_toplevel![instructions; Instruction::SetOutputRadix]],
            (ParserState::TopLevel, b'k') => incrementing![position;push_and_toplevel![instructions; Instruction::SetPrecision]],
            (ParserState::TopLevel, b'I') => incrementing![position;push_and_toplevel![instructions; Instruction::GetInputRadix]],
            (ParserState::TopLevel, b'O') => incrementing![position;push_and_toplevel![instructions; Instruction::GetOutputRadix]],
            (ParserState::TopLevel, b'K') => incrementing![position;push_and_toplevel![instructions; Instruction::GetPrecision]],
            (ParserState::TopLevel, b' ') => incrementing![position; ParserState::TopLevel], // do nothing
            (ParserState::TopLevel, b'\n') => incrementing![position; ParserState::TopLevel], // do nothing
            (ParserState::TopLevel, ch) => ParserState::Error(position, ParserErrorType::InvalidCharacter(ch)),
            (ParserState::Num{start, end, seen_dot: false}, b'.')  => incrementing![position; ParserState::Num{start, end: end + 1, seen_dot: true }], 
            (ParserState::Num{start, end, seen_dot}, b'0'...b'9') => incrementing![position; ParserState::Num{start, end: end + 1, seen_dot: seen_dot }], 
            (ParserState::Num{start, end, seen_dot: _seen_dot}, _) => push_and_toplevel![instructions; Instruction::Num(&program_text[start..end])],
            (ParserState::Register(register_operation_type), ch) => incrementing![
                position; 
                push_and_toplevel![ instructions; Instruction::RegisterOperation(register_operation_type, ch)]],
            (ParserState::Mark, b'>') => incrementing![position; ParserState::Register(RegisterOperationType::TosGeExecute)],
            (ParserState::Mark, b'<') => incrementing![position; ParserState::Register(RegisterOperationType::TosLeExecute)],
            (ParserState::Mark, b'=') => incrementing![position; ParserState::Register(RegisterOperationType::TosNeExecute)],
            (ParserState::Mark, _) => incrementing![position; ParserState::Command { start: position, end: position+1, }],
            (ParserState::Command { start, end }, b'\n') => incrementing![position; push_and_toplevel![instructions; Instruction::System(&program_text[start..end])]],
            (ParserState::Command { start, end }, _) => incrementing![position; ParserState::Command { start, end: end+1 }],
            (ParserState::ReadUntilByte{terminator, range: Range{start, end}}, ch) if ch == terminator => incrementing![
                position;
                push_and_toplevel![instructions; Instruction::System(&program_text[start .. end])]], 
            (ParserState::ReadUntilByte{terminator, range: Range{start, end}}, _) => incrementing![position; ParserState::ReadUntilByte{terminator, range: start .. end+1}],
        }
    }
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
