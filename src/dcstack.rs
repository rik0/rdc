use std::fmt;
use std::error;
use std::str;
use std::str::FromStr;
use std::io;
use num::Num;

use bigdecimal;
use bigdecimal::BigDecimal;
use num::bigint;
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

    #[allow(dead_code)]
    pub fn from_string_radix(
        s: &str,
        radix: u32,
    ) -> Result<MemoryCell, bigdecimal::ParseBigDecimalError> {
        Ok(MemoryCell::Num(BigDecimal::from_str_radix(s, radix)?))
    }

    #[allow(dead_code)]
    pub fn from_numstring(s: &str) -> Result<MemoryCell, bigdecimal::ParseBigDecimalError> {
        MemoryCell::from_string_radix(s, 10)
    }

    #[allow(dead_code)]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            &MemoryCell::Num(..) => None,
            &MemoryCell::Str(ref s) => Some(s),
        }
    }

    #[allow(dead_code)]
    pub fn num(self) -> Option<BigDecimal> {
        match self {
            MemoryCell::Str(..) => None,
            MemoryCell::Num(n) => Some(n),
        }
    }

    #[allow(dead_code)]
    pub fn to_str_radix(&self, radix: u32) -> String {
        match self {
            &MemoryCell::Num(ref n) => to_string_radix(n, radix),
            &MemoryCell::Str(ref v) => String::from_utf8(v.to_vec()).expect("internal utf8 error"),
        }
    }

    #[allow(dead_code)]
    pub fn apply_num<F>(&mut self, f: F) -> Result<Self, DCError>
    where
        F: Fn(&BigDecimal) -> BigDecimal,
    {
        match self {
            &mut MemoryCell::Num(ref n) => Ok(MemoryCell::Num(f(n))),
            &mut MemoryCell::Str(ref _v) => Err(DCError::NonNumericValue),
        }
    }

    pub fn apply_num_opt<F>(&mut self, f: F) -> Result<Self, DCError>
    where
        F: Fn(&BigDecimal) -> Option<BigDecimal>,
    {
        match self {
            &mut MemoryCell::Num(ref n) => Ok(MemoryCell::Num(
                f(n).ok_or(DCError::InternalConversionError)?,
            )),
            &mut MemoryCell::Str(ref _v) => Err(DCError::NonNumericValue),
        }
    }
}
#[test]
fn test_to_string_with_base() {
    assert_eq!(
        "10.2",
        MemoryCell::from_numstring("10.2").unwrap().to_str_radix(10)
    );
}

fn to_string_radix(n: &BigDecimal, radix: u32) -> String {
    if radix == 10 {
        return format!("{}", n);
    }
    let (bigint, exp) = n.as_bigint_and_exponent();
    let mut s = bigint.to_str_radix(radix);
    assert!(exp >= 0);
    if exp > 0 {
        let dot_insertion = s.len() - exp as usize;
        s.insert(dot_insertion, '.');
    }
    s
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
    NonStringValue,
    NumParseError,
    InternalConversionError,
}
static STACK_EMPTY: &'static str = "stack empty";
static NON_NUMERIC_VALUE: &'static str = "non numeric value";
static NON_STRING_VALUE: &'static str = "non string value";
static NUM_PARSE_ERROR: &'static str = "bytes do not represent a number";
static INTERNAL_CONVERSION_ERROR: &'static str = "internal conversion error";

impl DCError {
    pub fn message(&self) -> &'static str {
        match self {
            &DCError::StackEmpty => &STACK_EMPTY,
            &DCError::NonNumericValue => &NON_NUMERIC_VALUE,
            &DCError::NonStringValue => &NON_STRING_VALUE,
            &DCError::NumParseError => &NUM_PARSE_ERROR,
            &DCError::InternalConversionError => &INTERNAL_CONVERSION_ERROR,
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

fn make_big_decimal(digits: &[u8], radix: u32) -> Result<BigDecimal, DCError> {
    if let Some(n) = bigint::BigInt::parse_bytes(digits, radix) {
        return Ok(BigDecimal::new(n, 0));
    }
    Err(DCError::NumParseError)
}

#[derive(Debug)]
pub struct DCStack {
    stack: Vec<MemoryCell>,
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

    pub fn push_bytes_as_num(
        &mut self,
        integer: &[u8],
        fraction: &[u8],
        radix: u32,
    ) -> Result<(), DCError> {
        let scale = fraction.len();

        if cfg!(feature = "parse_all_bases") {
            println!(
                "slice {} {} {}",
                radix,
                String::from_utf8(Vec::from(integer)).unwrap(),
                String::from_utf8(Vec::from(fraction)).unwrap()
            );
            let mut integer = make_big_decimal(integer, radix)?;
            if scale > 0 {
                let mut fraction = make_big_decimal(fraction, radix)?;
                let radix = BigDecimal::from(radix);
                for _i in 1..scale {
                    fraction = fraction / &radix;
                }
                integer += fraction;
            }
            self.push(MemoryCell::from(integer));
            Ok(())
        } else {
            // TODO: 1. find a way to avoid copy and similar ops
            // 2. We can easily support other bases without digits
            // 3. it is a much better format to have one single buffer and
            //    the position, since it gives us the top flexibility
            // 4. We support "everything"

            if radix != 10 {
                panic!("only supporting base 10");
            }

            // TODO: now we must copy memory, but being slightly smarter with
            // the parser would avoid it (e.g., put all the bytes in a single slice)
            let mut v = Vec::with_capacity(integer.len() + fraction.len() + 1);
            v.extend_from_slice(integer);
            if !fraction.is_empty() {
                v.push(b'.');
                v.extend_from_slice(fraction);
            }

            self.stack.push(MemoryCell::Num(
                BigDecimal::parse_bytes(&v, 10).expect("number should have been valid"),
            ));
            Ok(())
        }
    }

    pub fn write_to<W>(&self, w: &mut W, radix: u32) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        for item in self.stack.iter().rev() {
            writeln!(w, "{}", item.to_str_radix(radix))?;
        }
        Ok(())
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

    #[allow(dead_code)]
    pub fn apply_tos_num<F>(&mut self, f: F) -> Result<(), DCError>
    where
        F: Fn(&BigDecimal) -> BigDecimal,
    {
        if self.is_empty() {
            return Err(DCError::StackEmpty);
        }
        let len = self.len();
        self.stack[len - 1] = self.stack[len - 1].apply_num(f)?;
        Ok(())
    }

    pub fn apply_tos_num_opt<F>(&mut self, f: F) -> Result<(), DCError>
    where
        F: Fn(&BigDecimal) -> Option<BigDecimal>,
    {
        if self.is_empty() {
            return Err(DCError::StackEmpty);
        }
        let len = self.len();
        self.stack[len - 1] = self.stack[len - 1].apply_num_opt(f)?;
        Ok(())
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

    pub fn pop_str(&mut self) -> Result<Vec<u8>, DCError> {
        match self.pop()? {
            MemoryCell::Num(n) => {
                self.stack.push(MemoryCell::Num(n));
                Err(DCError::NonStringValue)
            }
            MemoryCell::Str(s) => Ok(s),
        }
    }

    pub fn peek(&self) -> Result<&MemoryCell, DCError> {
        if self.len() > 0 {
            Ok(&self.stack[self.len() - 1])
        } else {
            Err(DCError::StackEmpty)
        }
    }

    pub fn peek_mut(&mut self) -> Result<&mut MemoryCell, DCError> {
        let len = self.len();
        if len > 0 {
            Ok(&mut self.stack[len - 1])
        } else {
            Err(DCError::StackEmpty)
        }
    }

    pub fn dup(&mut self) -> Result<(), DCError> {
        let tos = self.peek()?.clone();
        Ok(self.push(tos))
    }

    pub fn swap(&mut self) -> Result<(), DCError> {
        let former_tos = self.pop()?;
        match self.pop() {
            Ok(memory_cell) => {
                self.stack.push(former_tos);
                self.stack.push(memory_cell);
                Ok(())
            }
            Err(stack_error) => {
                self.stack.push(former_tos);
                Err(stack_error)
            }
        }
    }

    pub fn clear(&mut self) -> Result<(), DCError> {
        self.stack.clear();
        Ok(())
    }

    // pub fn appy_num<F, T>(&mut self, f: F) -> Result<T, DCError>
    // where
    //     F: Fn(&mut BigDecimal) -> T,
    // {
    //     match self.peek_mut()? {
    //         &mut MemoryCell::Num(ref mut n) => Ok(f(n)),
    //         &mut MemoryCell::Str(..) => Err(DCError::NonNumericValue),
    //     }
    // }
}

impl fmt::Display for DCStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for item in self.stack.iter().rev() {
            writeln!(f, "{}", item.to_str_radix(10))?;
        }
        Ok(())
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

#[test]
fn test_dup() {
    let mut s = dcstack_num!(10);
    assert_eq!(Ok(()), s.dup());
    assert_eq!(2, s.len());
}

#[test]
fn test_dup_empty() {
    let mut s = DCStack::new();
    assert_eq!(Err(DCError::StackEmpty), s.dup());
}

#[test]
fn test_swap_empty() {
    let mut s = DCStack::new();
    assert_eq!(Err(DCError::StackEmpty), s.swap());
}

#[test]
fn test_swap_one() {
    let mut s = dcstack_num!(10);
    assert_eq!(Err(DCError::StackEmpty), s.swap());
}

#[test]
fn test_swap() {
    use bigdecimal::ToPrimitive;
    let mut s = dcstack_num!(10, 20);
    assert_eq!(Ok(()), s.swap());
    assert_eq!(2, s.len());
    let a = s.pop_num().unwrap();
    let b = s.pop_num().unwrap();
    assert_eq!(20, ToPrimitive::to_u64(&b).unwrap());
    assert_eq!(10, ToPrimitive::to_u64(&a).unwrap());
}

#[test]
fn test_clear() {
    let mut s = dcstack_num!(10);
    assert_eq!(Ok(()), s.clear());
    assert_eq!(0, s.len());
}

#[test]
fn test_clear_empty() {
    let mut s = DCStack::new();
    assert_eq!(Ok(()), s.clear());
    assert_eq!(0, s.len());
}

#[test]
fn test_behavior() {
    let n = bigint::BigInt::parse_bytes(b"A", 16).expect("BigInt expecting success");

    //let n = BigDecimal::parse_bytes(b"A", 16).expect("BigDecimal expecting success");
    println!("{:?}", n);
    assert_eq!(n, bigint::BigInt::from(10));

    assert_eq!(
        bigint::BigInt::parse_bytes(b"1101", 2).expect("expecting result"),
        bigint::BigInt::from_str("13")
            .ok()
            .expect("expecting result 2")
    );

    let k = bigint::BigInt::parse_bytes(b"1234", 10).expect("really");
    assert_eq!(BigDecimal::new(k.clone(), 0), BigDecimal::from(1234));
    assert_eq!(BigDecimal::new(k.clone(), 1), BigDecimal::from(123.4));
}
