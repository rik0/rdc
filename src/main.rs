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
    Num{start: usize, end: usize, seen_dot: bool},
    End,
}

#[derive(Clone,Debug,PartialEq)]
struct ParserError {
    position: usize,
    error_type: ParserErrorType,
}

fn parse<T>(program_text: &[u8]) -> Result<Vec<Instruction<T>>, ParserError> {
    let mut state = ParserState::TopLevel;
    let mut instructions = Vec::new();
    let mut col: usize = 0;

    if program_text.len() == 0 {
        return Ok(instructions);
    }
    loop {
        if col >= program_text.len() {
            state = ParserState::End;
        }

        match state {
            ParserState::End => {
                break
            }
            ParserState::Error(_, _) => {
                break
            }
            ParserState::TopLevel => {
                let ch = program_text[col];

                match ch {
                    0 => instructions.push(Instruction::Nop),
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
                    b'i' => instructions.push(Instruction::SetInputRadix),
                    b'o' => instructions.push(Instruction::SetOutputRadix),
                    b'k' => instructions.push(Instruction::SetPrecision),
                    b'I' => instructions.push(Instruction::GetInputRadix),
                    b'O' => instructions.push(Instruction::GetOutputRadix),
                    b'K' => instructions.push(Instruction::GetPrecision),
                    ch => {
                        state = ParserState::Error(col, ParserErrorType::InvalidCharacter(ch));
                        continue;
                    }
                }

                col += 1;
            }
            ParserState::Num{start, end, seen_dot} => {
                let ch = program_text[col];
                match (seen_dot, ch) {
                    (true, b'.') => {

                    }
                    (false, b'.') => {

                    }
                    (_, b'0' ... b'9') => {


                    }
                    _ => {

                    }
                }

            }
        }
        
    }

    return match state {
        ParserState::Error(position, error_type) => Err(ParserError{position, error_type}),
        ParserState::End => Ok(instructions),
        ParserState::TopLevel => Err(ParserError{position: col, error_type: ParserErrorType::IllegalState("parsing stopped".to_string())}),
        _other => {

            Err(ParserError{position: col, error_type: ParserErrorType::IllegalState("not sure".to_string())})
        }
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
    print_test_p: ("p", Ok(vec![Instruction::PrintLN])),
    print_test_n: ("n", Ok(vec![Instruction::PrintPop])),
    print_test_p2: ("P", Ok(vec![Instruction::PrettyPrint])),
    print_test_f: ("f", Ok(vec![Instruction::PrintStack])),
    print_test_add: ("+", Ok(vec![Instruction::Add])),
    print_test_sub: ("-", Ok(vec![Instruction::Sub])),
    print_test_mul: ("*", Ok(vec![Instruction::Mul])),
    print_test_div: ("/", Ok(vec![Instruction::Div])),
    print_test_mod: ("%", Ok(vec![Instruction::Mod])),
    print_test_divmod: ("~", Ok(vec![Instruction::Divmod])),
    print_test_exp: ("^", Ok(vec![Instruction::Exp])),
    print_test_expmod: ("|", Ok(vec![Instruction::Modexp])),
    print_test_v: ("v", Ok(vec![Instruction::Sqrt])),
    print_test_c: ("c", Ok(vec![Instruction::Clear])),
    print_test_d: ("d", Ok(vec![Instruction::Dup])),
    print_test_r: ("r", Ok(vec![Instruction::Swap])),
    print_test_i: ("i", Ok(vec![Instruction::SetInputRadix])),
    print_test_o: ("o", Ok(vec![Instruction::SetOutputRadix])),
    print_test_k: ("k", Ok(vec![Instruction::SetPrecision])),
    print_test_i2: ("I", Ok(vec![Instruction::GetInputRadix])),
    print_test_o2: ("O", Ok(vec![Instruction::GetOutputRadix])),
    print_test_k2: ("K", Ok(vec![Instruction::GetPrecision])),
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
