use num;
use fmt;
use std::error;

#[derive(Clone, Debug, PartialEq)]
enum MemoryCell<'a, T> 
    where T : num::Num {
    Str(&'a [u8]),
    Num(T)
}

impl<'a, T: num::Num> MemoryCell<'a, T> {
    fn is_num(&self) -> bool {
        match self {
            &MemoryCell::Num(..) => true,
            &MemoryCell::Str(..) => false,
        }
    }

    // fn num() .. how can it work? sometimes the area would be consumed, sometimes it would not.
    // hey use ref!
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
pub struct DCStack<'a, T: num::Num> { 
    stack: Vec<MemoryCell<'a, T>>,
} 

macro_rules! dcstack {
    ( $ ( $ x : expr ) , * ) => ({
        let mut dcstack = DCStack::new();
        $( dcstack.push_num($x); )*
        dcstack
    })
}

impl<'a, T: num::Num> DCStack<'a, T> {
    pub fn new() -> DCStack<'a, T> {
        DCStack{stack: Vec::new()}
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn push_num(&mut self, item: T) {
        self.stack.push(MemoryCell::Num(item));
    }

    pub fn push_str(&mut self, item: &'a [u8]) {
        self.stack.push(MemoryCell::Str(item))
    }

    pub fn pop_num(&mut self) -> Result<T, DCError> {
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

    // fn pop(&mut self) -> Result<MemoryCell<T>, DCError> {
    //     match self.stack.pop() {
    //         Some(value) => Ok(value),
    //         None => ,
    //     }
    // }

}

#[test]
fn test_stack_empty_pop_num() {
    let mut s : DCStack<f64> = DCStack::new();
    assert_eq!(DCError::StackEmpty, s.pop_num().unwrap_err());
}


#[test]
fn test_stack_pop_num_num() {
    let mut s = dcstack![0];
    assert_eq!(0, s.pop_num().expect("i should not be empty"));
    assert!(s.is_empty());
}

