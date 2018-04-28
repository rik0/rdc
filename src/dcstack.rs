use num;
use fmt;

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

static STACK_EMPTY: &'static str = "stack empty";
static NON_NUMERIC_VALUE: &'static str = "non numeric value";

impl fmt::Display for DCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let message = match self {
            &DCError::StackEmpty => &STACK_EMPTY,
            &DCError::NonNumericValue => &NON_NUMERIC_VALUE,
        };
        write!(f, "{}", message)?;
        Ok(())
    }
}

pub struct DCStack<'a, T: num::Num> { 
    stack: Vec<MemoryCell<'a, T>>,
} 

macro_rules! dcstack {
    ( $ ( $ x : expr ) , * ) => {
        let mut dcstack = DCStack::new();
        $( dcstack.push($x); )*
        dcstack
    }
}

impl<'a, T: num::Num+Clone> DCStack<'a, T> {
    pub fn new() -> DCStack<'a, T> {
        DCStack{stack: Vec::new()}
    }

    pub fn pop_num(&mut self) -> Result<T, DCError> {
        
        let stack = &mut self.stack[..];
        match &mut stack[stack.len() -1] {
            &mut MemoryCell::Num(ref n) => Ok(n.clone()), // bad borrow
            &mut MemoryCell::Str(s) => {
                Err(DCError::NonNumericValue)
            }
        }
    }

        
    // fn pop_num(&mut self) -> Result<T, DCError> {
    //      match self.stack. {
    //     //     &mut MemoryCell::Num(ref n) => Ok(*n), // bad borrow
    //     //     &mut MemoryCell::Str(s) => {
    //     //         Err(DCError::NonNumericValue)
    //     //     }
    //     // }
    //      }
    // }

    fn pop_num2(&mut self) -> Result<T, DCError> {
        match self.stack.pop() {
            Some(MemoryCell::Num(n)) => Ok(n),
            Some(MemoryCell::Str(s)) => {
                // TODO 
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