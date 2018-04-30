pub type Register = u8;
//type ProgramText = &[u8];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RegisterOperationType {
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
    SetArray,
    GetArray,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction<'a> {
    Nop,
    Num(&'a [u8], &'a [u8]),
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
    OpToString,
    ExecuteTos,
    ExecuteInput,
    ReturnCaller,
    ReturnN,
    // status enquiry
    Digits,
    FractionDigits,
    StackDepth,
    // miscellaneous
    System(&'a [u8]),
    Comment(&'a [u8]),
}
