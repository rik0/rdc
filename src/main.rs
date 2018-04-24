
enum Cell<T> {
    N(T),
    S(String)
}

enum Instruction<T> {
    Nop,
    Num(T),
    // print
    PrintLN,
    PrintPop,
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

struct Program {

}

enum Parser {
    TopLevelParser,

}

struct ParserState<'a, T> {
    parser: Parser,
    instructions: Vec<Instruction<T>>,
    rest: &'a str
}

impl<'a, T> ParserState<'a, T> {
    fn parse_next(self) -> ParserState<'a, T> {
        return self
    }
}


fn parse<T>(program_text: &str) -> Program {
    let mut p = ParserState::<T>{ parser: Parser::TopLevelParser, instructions: Vec::new(), rest: program_text};

    while p.rest.len() > 0 {
        p = p.parse_next();
    }

    Program{}
}


fn main() {
    println!("Hello, world!");
}
