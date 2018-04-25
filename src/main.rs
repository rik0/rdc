use std::io::Write;

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
    Store,
    Load,
    StoreStack,
    LoadStack,
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
    TosGeExecute,
    TosGtExecute,
    TosLeExecute,
    TosLtExecute,
    TosEqExecute,
    TosNeExecute,
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
}

#[derive(Clone,Debug,PartialEq)]
enum ParserState {
    TopLevel,
    Error(usize, ParserErrorType),
    End,
}

#[derive(Clone,Debug,PartialEq)]
struct ParserError {
    position: usize,
    error_type: ParserErrorType,
}



fn byte_to_instruction<T>(ch: u8) -> Option<Instruction<T>> {
    match ch {
        0 => Some(Instruction::Nop),
        b'p' => Some(Instruction::PrintLN),
        b'n' => Some(Instruction::PrintPop),
        b'P' => Some(Instruction::PrettyPrint),
        b'f' => Some(Instruction::PrintStack),
        b'+' => Some(Instruction::Add),
        b'-' => Some(Instruction::Sub),
        b'*' => Some(Instruction::Mul),
        b'/' => Some(Instruction::Div),
        b'%' => Some(Instruction::Mod),
        b'~' => Some(Instruction::Divmod),
        b'^' => Some(Instruction::Exp),
        b'|' => Some(Instruction::Modexp),
        b'v' => Some(Instruction::Sqrt),
        b'c' => Some(Instruction::Clear),
        b'd' => Some(Instruction::Dup),
        b'r' => Some(Instruction::Swap),
        b'i' => Some(Instruction::SetInputRadix),
        b'o' => Some(Instruction::SetOutputRadix),
        b'k' => Some(Instruction::SetPrecision),
        b'I' => Some(Instruction::GetInputRadix),
        b'O' => Some(Instruction::GetOutputRadix),
        b'K' => Some(Instruction::GetPrecision),
        _ => None
    }
}


fn parse<T>(program_text: &[u8]) -> Result<Vec<Instruction<T>>, ParserError> {
    let mut state = ParserState::TopLevel;
    let mut instructions = Vec::new();
    let mut col: usize = 0;

    if program_text.len() == 0 {
        return Ok(instructions);
    }
    loop {
        if let ParserState::Error(position, error_type) = state {
            return Err(ParserError{position, error_type});
        }
        if col >= program_text.len() {
            state = ParserState::End;
            break;
        }

        let ch = program_text[col];
        match ch {
            0 => {
                match byte_to_instruction(ch) {
                    Some(instruction) => {
                        instructions.push(instruction);
                    },
                    None => {
                        state = ParserState::Error(col, ParserErrorType::IllegalState("error in byte_to_instruction or match".to_string()))
                    }
                }
            }
            ch => {
                state = ParserState::Error(col, ParserErrorType::InvalidCharacter(ch))
                
            }
        }

        col += 1;
    }

    return match state {
        ParserState::Error(position, error_type) => Err(ParserError{position, error_type}),
        ParserState::End => Ok(instructions),
        ParserState::TopLevel => Err(ParserError{position: col, error_type: ParserErrorType::IllegalState("parsing stopped".to_string())})
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
