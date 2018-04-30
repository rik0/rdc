use fmt;
use std::error;
use std::str::FromStr;

use bigdecimal;
use bigdecimal::BigDecimal;
use std::ops::*;

#[derive(Clone, Debug, PartialEq)]
pub enum MemoryCell {
    Str(Vec<u8>),
    Num(BigDecimal),
}

impl MemoryCell {
    pub fn is_num(&self) -> bool {
        match self {
            &MemoryCell::Num(..) => true,
            &MemoryCell::Str(..) => false,
        }
    }

    pub fn from_string(s: &str) -> MemoryCell {
        MemoryCell::Str(Vec::from(s))
    }

    // pub fn num(self) -> Option<BigDecimal> {
    //     match self {
    //         MemoryCell::Str(..) => None,
    //         MemoryCell::Num(n) => Some(n),
    //     }
    // }
}

impl AddAssign for MemoryCell {
    fn add_assign(&mut self, rhs: MemoryCell) {
        if let MemoryCell::Num(ref rhs) = rhs {
            if let &mut MemoryCell::Num(ref mut lhs) = self {
                lhs.add_assign(rhs);
            }
        }
    }
}

impl FromStr for MemoryCell {
    type Err = ();
    fn from_str(s: &str) -> Result<MemoryCell, Self::Err> {
        Ok(MemoryCell::from_string(s))
    }
}

impl<'a, T> From<T> for MemoryCell
where
    bigdecimal::BigDecimal: From<T>,
{
    fn from(n: T) -> MemoryCell {
        MemoryCell::Num(BigDecimal::from(n))
    }
}

#[test]
fn test_is_num() {
    assert!(MemoryCell::from(3).is_num());
    assert!(!MemoryCell::Str(Vec::from("a")).is_num());
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum DCError {
    StackEmpty,
    NonNumericValue,
    NumParseError,
}
static STACK_EMPTY: &'static str = "stack empty";
static NON_NUMERIC_VALUE: &'static str = "non numeric value";
static NUM_PARSE_ERROR: &'static str = "bytes do not represent a number";

impl DCError {
    pub fn message(&self) -> &'static str {
        match self {
            &DCError::StackEmpty => &STACK_EMPTY,
            &DCError::NonNumericValue => &NON_NUMERIC_VALUE,
            &DCError::NumParseError => &NUM_PARSE_ERROR,
        }
    }
}

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
pub struct DCStack {
    stack: Vec<MemoryCell>,
}

#[cfg(test)]
macro_rules! dcstack_num {
    ( $ ( $ x : expr ) , * ) => ({
        use dcstack;
        let mut dcstack = dcstack::DCStack::new();
        $( dcstack.push(dcstack::MemoryCell::from($x)); )*
        dcstack
    })
}

#[cfg(test)]
macro_rules! dcstack {
    ( $ ( $ x : expr ) , * ) => ({
        use dcstack;
        let mut dcstack = dcstack::DCStack::new();
        $( dcstack.push($x); )*
        dcstack
    })
}

impl DCStack {
    pub fn new() -> DCStack {
        DCStack { stack: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn push_num<U>(&mut self, item: U)
    where
        BigDecimal: From<U>,
    {
        self.push(MemoryCell::from(item))
    }

    pub fn push_bytes_as_num(&mut self, b: &[u8], radix: u32) -> Result<(), DCError> {
        if let Some(n) = BigDecimal::parse_bytes(b, radix) {
            self.push(MemoryCell::from(n));
            return Ok(());
        }
        Err(DCError::NumParseError)
    }

    pub fn binary_apply_and_consume_tos<F>(&mut self, f: F) -> Result<(), DCError>
    where
        F: Fn(&mut BigDecimal, BigDecimal),
    {
        let len = self.len();

        // we do prechecks here because dc does not consume the stack if there
        // are going to be issues of any kind:
        // $ dc -e '10 +f'
        // dc: stack empty
        // 10
        if len < 2 {
            return Err(DCError::StackEmpty);
        }

        if !self.stack[len - 1].is_num() || !self.stack[len - 2].is_num() {
            return Err(DCError::NonNumericValue);
        }

        let rhs = self.pop().unwrap();
        if let MemoryCell::Num(rhs) = rhs {
            if let &mut MemoryCell::Num(ref mut lhs) = self.peek_mut()? {
                f(lhs, rhs);
                Ok(())
            } else {
                Err(DCError::NonNumericValue)
            }
        } else {
            Err(DCError::NonNumericValue)
        }
    }

    pub fn push_str(&mut self, item: &[u8]) {
        self.push(MemoryCell::Str(Vec::from(item)))
    }

    pub fn push(&mut self, item: MemoryCell) {
        self.stack.push(item)
    }

    pub fn pop(&mut self) -> Result<MemoryCell, DCError> {
        if let Some(item) = self.stack.pop() {
            return Ok(item);
        }
        Err(DCError::StackEmpty)
    }

    pub fn pop_num(&mut self) -> Result<BigDecimal, DCError> {
        match self.pop()? {
            MemoryCell::Num(n) => Ok(n),
            MemoryCell::Str(s) => {
                // Slower than it should but it is only the error path
                // TODO make it faster
                self.stack.push(MemoryCell::Str(s));
                Err(DCError::NonNumericValue)
            }
        }
    }

    // pub fn peek(&self) -> Result<&MemoryCell, DCError> {
    //     if self.len() > 0 {
    //         Ok(&self.stack[self.len() - 1])
    //     } else {
    //         Err(DCError::StackEmpty)
    //     }
    // }

    pub fn peek_mut(&mut self) -> Result<&mut MemoryCell, DCError> {
        let len = self.len();
        if len > 0 {
            Ok(&mut self.stack[len - 1])
        } else {
            Err(DCError::StackEmpty)
        }
    }

    pub fn appy_num<F, T>(&mut self, f: F) -> Result<T, DCError>
    where
        F: Fn(&mut BigDecimal) -> T,
    {
        match self.peek_mut()? {
            &mut MemoryCell::Num(ref mut n) => Ok(f(n)),
            &mut MemoryCell::Str(..) => Err(DCError::NonNumericValue),
        }
    }

    pub fn clone_tos(&self) -> Result<MemoryCell, DCError> {
        if self.len() > 0 {
            Ok(self.stack[self.len() - 1].clone())
        } else {
            Err(DCError::StackEmpty)
        }
    }
}

#[test]
fn test_stack_empty_pop_num() {
    let mut s: DCStack = DCStack::new();
    assert_eq!(DCError::StackEmpty, s.pop_num().unwrap_err());
}

#[test]
fn test_stack_pop_num_num() {
    let mut s = dcstack_num![0];
    assert_eq!(
        BigDecimal::from(0),
        s.pop_num().expect("i should not be empty")
    );
    assert!(s.is_empty());
}

#[test]
fn test_push_pop() {
    use std::str::FromStr;
    let mut s: DCStack = DCStack::new();
    s.push_num(10.22);
    let bd = s.pop_num().expect("was expecting to get a number");
    assert_eq!(BigDecimal::from_str("10.22").expect("was a number"), bd);
}
