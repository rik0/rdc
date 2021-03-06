use std::ops::Range;
use std::fmt;
use std::error;

use instructions::*;

const STRING_TERMINATOR: u8 = b']';
const NEWLINE_BYTE: u8 = b'\n';

#[derive(Clone, Debug, PartialEq)]
enum Terminator {
    String,
    System,
    Comment,
}

impl PartialEq<u8> for Terminator {
    fn eq(&self, other: &u8) -> bool {
        match self {
            &Terminator::String => &STRING_TERMINATOR == other,
            &Terminator::System => &NEWLINE_BYTE == other,
            &Terminator::Comment => &NEWLINE_BYTE == other,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ParserErrorType {
    InvalidCharacter(u8),
    EOP(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserError<'a> {
    pub position: usize,
    error_type: ParserErrorType,
    pub program: Program<'a>,
    pub unparsed: &'a [u8],
}

static EOP_MESSAGE: &'static str = "end of stream";
static INVALID_CHARACTER: &'static str = "invalid character";

impl<'a> fmt::Display for ParserError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self.error_type {
            &ParserErrorType::InvalidCharacter(ch) => {
                write!(f, "'{}' ({:04o}) unimplemented", ch as char, ch)?;
            }
            &ParserErrorType::EOP(ref s) => {
                write!(f, "end of program {}", s)?;
            }
        }
        Ok(())
    }
}

impl<'a> error::Error for ParserError<'a> {
    fn description(&self) -> &str {
        match &self.error_type {
            &ParserErrorType::EOP(..) => &EOP_MESSAGE,
            &ParserErrorType::InvalidCharacter(..) => &INVALID_CHARACTER,
        }
    }
}

#[derive(Debug, PartialEq)]
enum ParserState {
    TopLevel,
    Error(usize, ParserErrorType),
    Num {
        start: usize,
        end: usize,
        dot_position: Option<usize>,
    },
    PrepareToReadUntil {
        terminator: Terminator,
    },
    ReadUntilByte {
        terminator: Terminator,
        range: Range<usize>,
    },
    Register(RegisterOperationType),
    Mark,
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

// TODO since apparently dc does process output fully even with parse errors (which are rare)
// we need to give ParserError some context to contain potentially the remaining characters to allow
// to restart parsing from those. See for example:
// dc -e '10]20f'
// dc: ']' (0135) unimplemented
// 20
// 10
pub fn parse(program_text: &[u8]) -> Result<Program, ParserError> {
    let mut state = ParserState::TopLevel;
    let mut program = Program::default();
    let mut position: usize = 0;

    loop {
        if position >= program_text.len() {
            return match state {
                ParserState::Error(position, error_type) => Err(ParserError {
                    position,
                    error_type,
                    program,
                    unparsed: &program_text[position + 1..],
                }),
                ParserState::TopLevel => Ok(program),
                ParserState::Num {
                    start,
                    end,
                    dot_position,
                } => {
                    let pos = dot_position.unwrap_or(end);
                   program.push(Instruction::Num(
                        &program_text[start..pos],
                        &program_text[::std::cmp::min(pos + 1, end)..end],
                    ));
                    Ok(program)
                }
                ParserState::Register(_) => Err(ParserError {
                    position,
                    error_type: ParserErrorType::EOP("was expecting a register".to_string()),
                    program,
                    unparsed: &program_text[position + 1..],
                }),
                ParserState::Mark => Ok(program),
                // dc actually seg faults in this case
                ParserState::PrepareToReadUntil {
                    terminator: Terminator::String,
                } => Err(ParserError {
                    position,
                    error_type: ParserErrorType::EOP("string not completed".to_string()),
                    program,
                    unparsed: &program_text[position + 1..],
                }),
                ParserState::PrepareToReadUntil { .. } => Ok(program),
                ParserState::ReadUntilByte {
                    terminator: Terminator::String,
                    range,
                } => {
                    program.push(Instruction::Str(&program_text[range]));
                    Ok(program)
                }
                ParserState::ReadUntilByte {
                    terminator: Terminator::System,
                    range,
                } => {
                    program.push(Instruction::System(&program_text[range]));
                    Ok(program)
                }
                ParserState::ReadUntilByte {
                    terminator: Terminator::Comment,
                    range,
                } => {
                    program.push(Instruction::Comment(&program_text[range]));
                    Ok(program)
                }
            };
        }
        state = match (state, program_text[position]) {
            (ParserState::Error(position, error_type), _) => {
                return Err(ParserError {
                    position,
                    error_type,
                    program,
                    unparsed: &program_text[position + 1..],
                })
            }
            (ParserState::TopLevel, 0) => {
                incrementing![position; push_and_toplevel![program; Instruction::Nop]]
            }
            (ParserState::TopLevel, b'.') => incrementing![position; ParserState::Num {
                    start: position,
                    end: position + 1,
                    dot_position: Some(position),
            }],
            (ParserState::TopLevel, b'0'...b'9') => incrementing![position; ParserState::Num {
                    start: position,
                    end: position + 1,
                    dot_position: None,
            }],
            (ParserState::TopLevel, b'A'...b'F') => incrementing![position; ParserState::Num {
                    start: position,
                    end: position + 1,
                    dot_position: None,
            }],
            (ParserState::TopLevel, b'p') => {
                incrementing![position; push_and_toplevel![program; Instruction::PrintLN]]
            }
            (ParserState::TopLevel, b'n') => {
                incrementing![position; push_and_toplevel![program; Instruction::PrintPop]]
            }
            (ParserState::TopLevel, b'P') => {
                incrementing![position; push_and_toplevel![program; Instruction::PrettyPrint]]
            }
            (ParserState::TopLevel, b'f') => {
                incrementing![position; push_and_toplevel![program; Instruction::PrintStack]]
            }
            (ParserState::TopLevel, b'+') => {
                incrementing![position; push_and_toplevel![program; Instruction::Add]]
            }
            (ParserState::TopLevel, b'-') => {
                incrementing![position; push_and_toplevel![program; Instruction::Sub]]
            }
            (ParserState::TopLevel, b'*') => {
                incrementing![position; push_and_toplevel![program; Instruction::Mul]]
            }
            (ParserState::TopLevel, b'/') => {
                incrementing![position; push_and_toplevel![program; Instruction::Div]]
            }
            (ParserState::TopLevel, b'%') => {
                incrementing![position; push_and_toplevel![program; Instruction::Mod]]
            }
            (ParserState::TopLevel, b'~') => {
                incrementing![position; push_and_toplevel![program; Instruction::Divmod]]
            }
            (ParserState::TopLevel, b'^') => {
                incrementing![position; push_and_toplevel![program; Instruction::Exp]]
            }
            (ParserState::TopLevel, b'|') => {
                incrementing![position; push_and_toplevel![program; Instruction::Modexp]]
            }
            (ParserState::TopLevel, b'v') => {
                incrementing![position; push_and_toplevel![program; Instruction::Sqrt]]
            }
            (ParserState::TopLevel, b'c') => {
                incrementing![position; push_and_toplevel![program; Instruction::Clear]]
            }
            (ParserState::TopLevel, b'd') => {
                incrementing![position;push_and_toplevel![program; Instruction::Dup]]
            }
            (ParserState::TopLevel, b'r') => {
                incrementing![position;push_and_toplevel![program; Instruction::Swap]]
            }
            (ParserState::TopLevel, b's') => {
                incrementing![position;ParserState::Register(RegisterOperationType::Store)]
            }
            (ParserState::TopLevel, b'l') => {
                incrementing![position;ParserState::Register(RegisterOperationType::Load)]
            }
            (ParserState::TopLevel, b'S') => {
                incrementing![position;ParserState::Register(RegisterOperationType::StoreStack)]
            }
            (ParserState::TopLevel, b'L') => {
                incrementing![position;ParserState::Register(RegisterOperationType::LoadStack)]
            }
            (ParserState::TopLevel, b'>') => {
                incrementing![position;ParserState::Register(RegisterOperationType::TosGtExecute)]
            }
            (ParserState::TopLevel, b'<') => {
                incrementing![position;ParserState::Register(RegisterOperationType::TosLtExecute)]
            }
            (ParserState::TopLevel, b'=') => {
                incrementing![position;ParserState::Register(RegisterOperationType::TosEqExecute)]
            }
            (ParserState::TopLevel, b'!') => incrementing![position;ParserState::Mark],
            (ParserState::TopLevel, b'i') => {
                incrementing![position;push_and_toplevel![program; Instruction::SetInputRadix]]
            }
            (ParserState::TopLevel, b'o') => {
                incrementing![position;push_and_toplevel![program; Instruction::SetOutputRadix]]
            }
            (ParserState::TopLevel, b'k') => {
                incrementing![position;push_and_toplevel![program; Instruction::SetPrecision]]
            }
            (ParserState::TopLevel, b'I') => {
                incrementing![position;push_and_toplevel![program; Instruction::GetInputRadix]]
            }
            (ParserState::TopLevel, b'O') => {
                incrementing![position;push_and_toplevel![program; Instruction::GetOutputRadix]]
            }
            (ParserState::TopLevel, b'K') => {
                incrementing![position;push_and_toplevel![program; Instruction::GetPrecision]]
            }
            (ParserState::TopLevel, b'[') => {
                incrementing![position; ParserState::PrepareToReadUntil{terminator: Terminator::String}]
            }
            (ParserState::TopLevel, b'a') => {
                incrementing![position;push_and_toplevel![program; Instruction::OpToString]]
            }
            (ParserState::TopLevel, b'x') => {
                incrementing![position;push_and_toplevel![program; Instruction::ExecuteTos]]
            }
            (ParserState::TopLevel, b'Z') => {
                incrementing![position;push_and_toplevel![program; Instruction::Digits]]
            }
            (ParserState::TopLevel, b'X') => {
                incrementing![position;push_and_toplevel![program; Instruction::FractionDigits]]
            }
            (ParserState::TopLevel, b'z') => {
                incrementing![position;push_and_toplevel![program; Instruction::StackDepth]]
            }
            (ParserState::TopLevel, b'#') => {
                incrementing![position; ParserState::PrepareToReadUntil{terminator: Terminator::Comment}]
            }
            (ParserState::TopLevel, b':') => {
                incrementing![position;ParserState::Register(RegisterOperationType::SetArray)]
            }
            (ParserState::TopLevel, b';') => {
                incrementing![position;ParserState::Register(RegisterOperationType::GetArray)]
            }
            (ParserState::TopLevel, b'?') => {
                incrementing![position;push_and_toplevel![program; Instruction::ExecuteInput]]
            }
            (ParserState::TopLevel, b'q') => {
                incrementing![position;push_and_toplevel![program; Instruction::ReturnCaller]]
            }
            (ParserState::TopLevel, b'Q') => {
                incrementing![position;push_and_toplevel![program; Instruction::ReturnN]]
            }
            (ParserState::TopLevel, b' ') => incrementing![position; ParserState::TopLevel], // do nothing
            (ParserState::TopLevel, b'\n') => incrementing![position; ParserState::TopLevel], // do nothing
            (ParserState::TopLevel, ch) => {
                incrementing![position; ParserState::Error(position, ParserErrorType::InvalidCharacter(ch))]
            }

            (
                ParserState::Num {
                    start,
                    end,
                    dot_position: None,
                },
                b'.',
            ) => {
                incrementing![position; ParserState::Num{start, end: end + 1, dot_position: Some(position) }]
            }
            (
                ParserState::Num {
                    start,
                    end,
                    dot_position,
                },
                b'0'...b'9',
            ) => {
                incrementing![position; ParserState::Num{start, end: end + 1, dot_position: dot_position }]
            }
            (
                ParserState::Num {
                    start,
                    end,
                    dot_position,
                },
                b'A'...b'F',
            ) => {
                incrementing![position; ParserState::Num{start, end: end + 1, dot_position: dot_position }]
            }
            (
                ParserState::Num {
                    start,
                    end,
                    dot_position,
                },
                _,
            ) => {
                let dot_pos = dot_position.unwrap_or(end);
                push_and_toplevel![program; Instruction::Num(&program_text[start..dot_pos], &program_text[::std::cmp::min(dot_pos+1, end)..end])]
            }
            (ParserState::Register(register_operation_type), ch) => incrementing![
                position; 
                push_and_toplevel![ program; Instruction::RegisterOperation(register_operation_type, ch)]],
            (ParserState::Mark, b'>') => {
                incrementing![position; ParserState::Register(RegisterOperationType::TosGeExecute)]
            }
            (ParserState::Mark, b'<') => {
                incrementing![position; ParserState::Register(RegisterOperationType::TosLeExecute)]
            }
            (ParserState::Mark, b'=') => {
                incrementing![position; ParserState::Register(RegisterOperationType::TosNeExecute)]
            }
            (ParserState::Mark, _) => {
                incrementing![position; ParserState::ReadUntilByte { terminator: Terminator::System, range: position .. position+1 }]
            }
            (ParserState::PrepareToReadUntil { ref terminator }, ch) if *terminator == ch => {
                incrementing![position; push_and_toplevel![program; Instruction::Str(&program_text[position..position])]]
            }
            (ParserState::PrepareToReadUntil { terminator }, _) => {
                incrementing![position; ParserState::ReadUntilByte{terminator, range: position..position+1}]
            }
            (
                ParserState::ReadUntilByte {
                    terminator: Terminator::System,
                    range: Range { start, end },
                },
                NEWLINE_BYTE,
            ) => incrementing![
                position;
                push_and_toplevel![program; Instruction::System(&program_text[start .. end])]],
            (
                ParserState::ReadUntilByte {
                    terminator: Terminator::String,
                    range: Range { start, end },
                },
                STRING_TERMINATOR,
            ) => incrementing![
                position;
                push_and_toplevel![program; Instruction::Str(&program_text[start .. end])]],
            (
                ParserState::ReadUntilByte {
                    terminator: Terminator::Comment,
                    range: Range { start, end },
                },
                NEWLINE_BYTE,
            ) => incrementing![
                position;
                push_and_toplevel![program; Instruction::Comment(&program_text[start .. end])]],
            (
                ParserState::ReadUntilByte {
                    terminator,
                    range: Range { start, end },
                },
                _,
            ) => {
                incrementing![position; ParserState::ReadUntilByte{terminator, range: start .. end+1}]
            }
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
            assert_eq!(expected, parse(input.as_bytes()).map(|p| p.instructions));
        }
    )*
    }
}

parse_tests! {
    parse_test_empty: ("", Ok(vec![])),
    // parse_test_invalid: ("\x01", Err(ParserError{position: 0, error_type: ParserErrorType::InvalidCharacter(1), instructions: Vec::new(), unparsed: &"\x01"[..]})),
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
    parse_test_0: ("0", Ok(vec![Instruction::Num("0".as_bytes(), "".as_bytes())])),
    parse_test_0dot: ("0.", Ok(vec![Instruction::Num("0".as_bytes(), "".as_bytes())])),
    parse_test_dot0: (".0", Ok(vec![Instruction::Num("".as_bytes(), "0".as_bytes())])),
    parse_test_132763: ("132763", Ok(vec![Instruction::Num("132763".as_bytes(), "".as_bytes())])),
    parse_test_1: ("1", Ok(vec![Instruction::Num("1".as_bytes(), "".as_bytes())])),
    parse_test_1dot: ("1.", Ok(vec![Instruction::Num("1".as_bytes(), "".as_bytes())])),
    parse_test_dot1: (".1", Ok(vec![Instruction::Num("".as_bytes(), "1".as_bytes())])),
    parse_test_dot: (".", Ok(vec![Instruction::Num("".as_bytes(), "".as_bytes())])),
    parse_test_dotdot: ("..", Ok(vec![Instruction::Num("".as_bytes(), "".as_bytes()), Instruction::Num("".as_bytes(), "".as_bytes())])),
    parse_test_dot_dot: (". .", Ok(vec![Instruction::Num("".as_bytes(), "".as_bytes()), Instruction::Num("".as_bytes(), "".as_bytes())])),
    parse_test_zero_dot_zero: ("0.0", Ok(vec![Instruction::Num("0".as_bytes(), "0".as_bytes())])),
    parse_test_00: ("00", Ok(vec![Instruction::Num("00".as_bytes(), "".as_bytes())])),
    parse_test_11: ("11", Ok(vec![Instruction::Num("11".as_bytes(), "".as_bytes())])),
    parse_test_a2_dot_1: ("A.1", Ok(vec![Instruction::Num("A".as_bytes(), "1".as_bytes())])),
    parse_test_0_0: ("0 0", Ok(vec![Instruction::Num("0".as_bytes(), "".as_bytes()), Instruction::Num("0".as_bytes(), "".as_bytes())])),
    parse_test_1_1: ("1 1", Ok(vec![Instruction::Num("1".as_bytes(), "".as_bytes()), Instruction::Num("1".as_bytes(), "".as_bytes())])),
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
    parse_test_sysa10: ("!a\n10", Ok(vec![Instruction::System("a".as_bytes()), Instruction::Num("10".as_bytes(), "".as_bytes())])),
    parse_test_ltagt: ("<>", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::TosLtExecute, b'>' as Register)])),
    parse_test_str_aa3: ("[aa]3", Ok(vec![Instruction::Str("aa".as_bytes()), Instruction::Num("3".as_bytes(), "".as_bytes())])),
    parse_test_str_aa: ("[aa]", Ok(vec![Instruction::Str("aa".as_bytes())])),
    parse_test_str_empty: ("[]", Ok(vec![Instruction::Str(&[])])),
    parse_test_str_aanl: ("[aa\n]", Ok(vec![Instruction::Str("aa\n".as_bytes())])),
    parse_test_str_quoteaanl: ("[!aa\n]", Ok(vec![Instruction::Str("!aa\n".as_bytes())])),
    parse_test_str_aa_not_term: ("[aa", Ok(vec![Instruction::Str("aa".as_bytes())])),
    parse_test_a: ("a", Ok(vec![Instruction::OpToString])),
    parse_test_z2: ("Z", Ok(vec![Instruction::Digits])),
    parse_test_x2: ("X", Ok(vec![Instruction::FractionDigits])),
    parse_test_z: ("z", Ok(vec![Instruction::StackDepth])),
    parse_test_comment1: ("10 # foo 20", Ok(vec![Instruction::Num("10".as_bytes(), "".as_bytes()), Instruction::Comment(" foo 20".as_bytes())])),
    parse_test_comment2: ("10 # foo\n20", Ok(vec![Instruction::Num("10".as_bytes(), "".as_bytes()), Instruction::Comment(" foo".as_bytes()), Instruction::Num("20".as_bytes(), "".as_bytes())])),
    parse_test_set_array: (":a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::SetArray, b'a' as Register)])),
    parse_test_get_array: (";a", Ok(vec![Instruction::RegisterOperation(RegisterOperationType::GetArray, b'a' as Register)])),
    parse_test_input: ("?", Ok(vec![Instruction::ExecuteInput])),
    parse_test_return_caller: ("q", Ok(vec![Instruction::ReturnCaller])),
    parse_test_returnn: ("Q", Ok(vec![Instruction::ReturnN])),

    // add failure tests
}


#[test]
fn testparse_invalid() {
    let input = b"\x01";
    if let Err(parse_error) = parse(input) {
        assert_eq!(ParserErrorType::InvalidCharacter(1), parse_error.error_type);
        assert_eq!(0, parse_error.position);
        assert!(parse_error.unparsed.is_empty());
        assert!(parse_error.program.instructions.is_empty())
    } else {
        assert!(false)
    }
}


macro_rules! parse_reversibility {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        #[allow(non_snake_case)]
        fn $name() {
            let input = $value;

            if let Ok(Program{instructions}) = parse(input.as_bytes()) {
                let regenerated: String =instructions.iter().map(|i| format!("{}", i)).collect::<String>();
                assert_eq!(input, regenerated);
            } else {
                assert!(false);
            }
        }
    )*
    }
}

parse_reversibility! {
    rev_empty: "",
    zero: "\0",
    n: "123",
    pln: "p",
    pln2: "P",
    pln3: "n",
    ops: "+-*/%^|v",
    stack: "cdr",
    sa: "sa",
    la: "la",
    Sa: "Sa",
    La: "La",
    radix_and_prexision: "iokIOK",
    more: "ax?qQZXz",
    comment: "#foo\n",
    command: "!bar\n",
}