use std::io::Write;
use std::str;
use std::str::FromStr;
use std::error;
use std::error::Error;
use std::f64;


type Register = u8;




#[derive(Copy, Clone, Debug,PartialEq)]
enum Instruction<T> {
    Nop,
    Num(T),
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
    Store(Register),
    Load(Register),
    StoreStack(Register),
    LoadStack(Register),
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
    TosGeExecute(Register),
    TosGtExecute(Register),
    TosLeExecute(Register),
    TosLtExecute(Register),
    TosEqExecute(Register),
    TosNeExecute(Register),
    ExecuteInput,
    ReturnCaller,
    ReturnN,
    // status enquiry
    Digits,
    FractionDigits,
    StackDepth,
    // miscellaneous
    System,
    Comment,
    SetArray,
    GetArray
}

#[derive(Clone,Debug,PartialEq)]
enum ParserErrorType {
    IllegalState(String),
    InvalidCharacter(u8),
    NumParseError(usize, usize, String),
    EOP(String),
}

#[derive(Clone,Debug,PartialEq)]
enum ParserState {
    TopLevel,
    Error(usize, ParserErrorType),
    Num{start: usize, end: usize, seen_dot: bool},
    Register(u8),
    End,
}

#[derive(Clone,Debug,PartialEq)]
struct ParserError {
    position: usize,
    error_type: ParserErrorType,
}

fn parse<T>(program_text: &[u8]) -> Result<Vec<Instruction<T>>, ParserError> 
    where T: FromStr + Default,
          <T as str::FromStr>::Err: error::Error
{
    let mut state = ParserState::TopLevel;
    let mut instructions = Vec::new();
    let mut position: usize = 0;

    if program_text.len() == 0 {
        return Ok(instructions);
    }
    loop {
        match state {
            ParserState::End => {
                break
            }
            ParserState::Error(_, _) => {
                break
            }
            ParserState::TopLevel => {
                if position >= program_text.len() {
                    break
                }
                let ch = program_text[position];

                match ch {
                    0 => instructions.push(Instruction::Nop),
                    b'.' => {
                        // here we effectively consume one character, so we must go through the increment
                        state = ParserState::Num{start: position, end: position+1, seen_dot: true};
                    }
                    b'0'...b'9' => {
                        // here we effectively consume one character, so we must go through the increment
                        state = ParserState::Num{start: position, end: position+1, seen_dot: false};
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
                    b's'|b'l'|b'S'|b'L'|b'>'|b'<'|b'=' => {
                        // consumed char for operations, move to next position to get the register
                        state = ParserState::Register(ch)
                    }
                    b'i' => instructions.push(Instruction::SetInputRadix),
                    b'o' => instructions.push(Instruction::SetOutputRadix),
                    b'k' => instructions.push(Instruction::SetPrecision),
                    b'I' => instructions.push(Instruction::GetInputRadix),
                    b'O' => instructions.push(Instruction::GetOutputRadix),
                    b'K' => instructions.push(Instruction::GetPrecision),
                    b' '|b'\n' => (), // do nothing
                    ch => {
                        state = ParserState::Error(position, ParserErrorType::InvalidCharacter(ch));
                        continue;
                    }
                }

                position += 1;
            }
            ParserState::Num{start, end, seen_dot} => {
                if position >= program_text.len() {
                    let chars = &program_text[start..end];

                    match ascii_to_num::<T>(chars) {
                        Ok(n) => {
                            instructions.push(Instruction::Num(n));
                            state = ParserState::End;
                        }
                        Err(reason) => {
                            state = ParserState::Error(position, ParserErrorType::NumParseError(start, end, reason));
                        }
                    }
                    break

                }
                let ch = program_text[position];
                match (seen_dot, ch) {
                    (false, b'.') => {
                        // if we are here, we were alredy building a number and finally we got the .
                        state = ParserState::Num{start, end: end+1, seen_dot: true};
                        position += 1;
                    }
                    (_, b'0' ... b'9') => {
                        state = ParserState::Num{start, end: end+1, seen_dot: true};
                        position += 1;
                    }
                    (true, b'.')|_ => {
                        // it means it initiates a new number, store the old and start again
                        // we must not advance the position: note that this means we are looping
                        // a bit more than necessary, but it makes the logic simpler
                        let chars = &program_text[start..end];

                        match ascii_to_num::<T>(chars) {
                            Ok(n) => {
                                instructions.push(Instruction::Num(n));
                                state = ParserState::TopLevel;
                            }
                            Err(reason) => {
                                state = ParserState::Error(position, ParserErrorType::NumParseError(start, end, reason))
                            }
                        }
                    }
                }
            }
            ParserState::Register(opbyte) => {
                if position >= program_text.len() {
                    state = ParserState::Error(position, ParserErrorType::EOP("was expecting a register".to_string()));
                    break
                }
                let ch = program_text[position];


                position += 1;

            }
        }
    }

    return match state {
        ParserState::Error(position, error_type) => Err(ParserError{position, error_type}),
        ParserState::End => Ok(instructions),
        //ParserState::TopLevel => Err(ParserError{position: position, error_type: ParserErrorType::IllegalState("parsing stopped".to_string())}),
        ParserState::TopLevel => Ok(instructions),
        _other => {

            Err(ParserError{position: position, error_type: ParserErrorType::IllegalState("not sure".to_string())})
        }
    }
}

fn register_operation<T>(opbyte: u8, register_byte: u8) -> Result<Instruction<T>, ParserErrorType> {
    let r : Register = register_byte as Register;

    return match opbyte {
        b's' => Ok(Instruction::Store(r)),
        b'l' => Ok(Instruction::Load(r)),
        b'S' => Ok(Instruction::StoreStack(r)),
        b'L' => Ok(Instruction::LoadStack(r)),
        b'<' => Ok(Instruction::TosLtExecute(r)),
        b'>' => Ok(Instruction::TosGtExecute(r)),
        b'=' => Ok(Instruction::TosEqExecute(r)),
        ch => Err(ParserErrorType::IllegalState("there is an issue in register_operation or in the RegisterParse".to_string()))
    }
}


fn ascii_to_num<T>(bytes: &[u8]) -> Result<T, String> 
    where T: FromStr + Default,
          <T as str::FromStr>::Err: error::Error
{
    return match str::from_utf8(bytes) {
        Ok(".") => Ok(T::default()),
        Ok(chars) => {
            match T::from_str(chars) {
                Ok(n) => Ok(n),
                Err(error) => Err(error.description().to_string()),
            }
        }
        Err(utf8error) => Err(utf8error.description().to_string())
    }
}

macro_rules! parse_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (input, expected) = $value;
            assert_eq!(expected, parse::<f64>(input.as_bytes()));
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
    parse_test_0: ("0", Ok(vec![Instruction::Num(0.0)])),
    parse_test_1: ("1", Ok(vec![Instruction::Num(1.0)])),
    parse_test_dot: (".", Ok(vec![Instruction::Num(0.0)])),
    parse_test_dotdot: ("..", Ok(vec![Instruction::Num(0.0), Instruction::Num(0.0)])),
    parse_test_dot_dot: (". .", Ok(vec![Instruction::Num(0.0), Instruction::Num(0.0)])),
    parse_test_dot0: (".0", Ok(vec![Instruction::Num(0.0)])),
    parse_test_0dot: ("0.", Ok(vec![Instruction::Num(0.0)])),
    parse_test_0dot0: ("0.0", Ok(vec![Instruction::Num(0.0)])),
    parse_test_dot1: (".1", Ok(vec![Instruction::Num(0.1)])),
    parse_test_1dot: ("1.", Ok(vec![Instruction::Num(1.0)])),
    parse_test_1dot1: ("1.1", Ok(vec![Instruction::Num(1.1)])),
    parse_test_dot1dot1: (".1.1", Ok(vec![Instruction::Num(0.1), Instruction::Num(0.1)])),
    parse_test_all_digits: ("1234567890", Ok(vec![Instruction::Num(1234567890.0)])),
    parse_test_00: ("00", Ok(vec![Instruction::Num(0.0)])),
    parse_test_11: ("11", Ok(vec![Instruction::Num(11.0)])),
    parse_test_0_0: ("0 0", Ok(vec![Instruction::Num(0.0), Instruction::Num(0.0)])),
    parse_test_1_1: ("1 1", Ok(vec![Instruction::Num(1.0), Instruction::Num(1.0)])),
}

#[test]
fn testparse() {
    let (input, expected) = ("", Ok(vec![]));
    assert_eq!(expected, parse::<f64>(input.as_bytes()));
}


fn main() {
    match parse::<f64>(&("".as_bytes())) {
        Ok(instructions) => { println!("{:?}", instructions) },
        Err(ParserError{position, error_type}) => {
            writeln!(std::io::stderr(), "{:?} at col {}", error_type, position).unwrap();
            //std::process:exit(1);
        }
    }
    println!("Hello, world!");
}

// impl<'a, T> Parser<'a, T> {
//     fn parse(mut &self) {
//         while self.col < self.program_text.len()  {
//             match self.parser {
//                 ParserState::TopLevel => {
//                     let current = self.program_text[self.col];
//                     match current {
//                         0 => {
//                             match byte_to_instruction(current) {
//                                 Some(instruction) => {
//                                     self.instructions.push(instruction);
//                                     self.col += 1;
//                                 }
//                                 None => {
//                                     self.parser = ParserState::Error(self.col, ErrorType::IllegalState);
//                                 }
//                             }
//                         }
//                         _ => {

//                         }
//                     }

//                 }
//                 ParserState::Error(_, _) => {
//                     return 
//                 }
//                 ParserState::End => {
//                     return
//                 }
//             }
//         }
//     }
// }
