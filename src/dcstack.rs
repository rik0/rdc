use num;
use fmt;
use std::error;

use num::bigint::BigInt;
use bigdecimal::BigDecimal;

#[derive(Clone, Debug, PartialEq)]
enum MemoryCell<'a> {
    Str(&'a [u8]),
    Num(BigDecimal)
}

impl<'a> MemoryCell<'a> {
    fn is_num(&self) -> bool {
        match self {
            &MemoryCell::Num(..) => true,
            &MemoryCell::Str(..) => false,
        }
    }
}

impl<'a> From<u64> for MemoryCell<'a> {
    fn from(n: u64) -> MemoryCell<'a> {
        MemoryCell::Num(BigDecimal::new(BigInt::from(n), 0))
    }
}

#[test]
fn test_is_num() {
    assert!(MemoryCell::from(3).is_num());
    assert!(!MemoryCell::Str("a".as_bytes()).is_num());
}


#[derive(Clone, Debug, Copy, PartialEq)]
pub enum DCError {
    StackEmpty,
    NonNumericValue,
}

impl DCError {
    pub fn message(&self) -> &'static str {
        match self {
            &DCError::StackEmpty => &STACK_EMPTY,
            &DCError::NonNumericValue => &NON_NUMERIC_VALUE,
        }
    }
}

static STACK_EMPTY: &'static str = "stack empty";
static NON_NUMERIC_VALUE: &'static str = "non numeric value";

impl fmt::Display for DCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message())?;
        Ok(())
    }
}

impl error::Error for DCError {
    fn description(&self) -> &str {
        self.message()
    }
}

#[derive(Debug)]
pub struct DCStack<'a> { 
    stack: Vec<MemoryCell<'a>>,
} 

macro_rules! dcstack {
    ( $ ( $ x : expr ) , * ) => ({
        let mut dcstack = DCStack::new();
        $( dcstack.push(MemoryCell::from($x)); )*
        dcstack
    })
}

impl<'a> DCStack<'a> {
    pub fn new() -> DCStack<'a> {
        DCStack{stack: Vec::new()}
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn push_num(&mut self, item: BigDecimal) {
        self.push(MemoryCell::Num(item));
    }

    pub fn push_str(&mut self, item: &'a [u8]) {
        self.push(MemoryCell::Str(item))
    }

    pub fn push(&mut self, item: MemoryCell<'a>) {
        self.stack.push(item)
    }

    pub fn pop_num(&mut self) -> Result<BigDecimal, DCError> {
        match self.stack.pop() {
            Some(MemoryCell::Num(n)) => Ok(n),
            Some(MemoryCell::Str(s)) => {
                // Slower than it should but it is only the error path
                // TODO make it faster
                self.stack.push(MemoryCell::Str(s));
                Err(DCError::NonNumericValue)
            }
            None => Err(DCError::StackEmpty),
        }
    }
}

#[test]
fn test_stack_empty_pop_num() {
    let mut s : DCStack = DCStack::new();
    assert_eq!(DCError::StackEmpty, s.pop_num().unwrap_err());
}


#[test]
fn test_stack_pop_num_num() {
    let mut s = dcstack![0];
    assert_eq!(BigDecimal::from(0), s.pop_num().expect("i should not be empty"));
    assert!(s.is_empty());
}

