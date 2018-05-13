use num::{self, ToPrimitive};
use std::borrow::Cow;
use std::fmt::{self, Display};
use std::cmp::{self, Ordering};
use std::error;
use std::str::FromStr;

use std::f32;

// #[derive(Copy, PartialEq, Debug, Clone)]
// enum Sign {
//     Plus,
//     Minus
// }

#[derive(Clone, Debug, PartialEq)]
enum ParseDCNumberErrorKind {
    EmptyString,
    RepeatedDot,
    InvalidDigit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParseDCNumberError {
    kind: ParseDCNumberErrorKind
}

impl ParseDCNumberError {
    fn message(&self) -> &'static str {
        match self.kind {
            ParseDCNumberErrorKind::EmptyString => &"empty string",
            ParseDCNumberErrorKind::RepeatedDot => &"repeated dot",
            ParseDCNumberErrorKind::InvalidDigit => &"invalid digit",
        }
    }
}

impl Display for ParseDCNumberError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.message())?;
        Ok(())
    }
}

impl error::Error for ParseDCNumberError {
    fn description(&self) -> &str {
        self.message()
    }
}




#[derive(Clone, Debug)]
pub struct UnsignedDCNumber<'a> {
    // TODO: maybe use nibble?
    // digits are in BigEndian
    digits: Cow<'a, [u8]>,
    separator: usize,
}

macro_rules! static_unsigned_dcnumber {
    ($dcnumber_name:ident; $digits_name:ident : $digits_type:ty = $digits:expr) => {
        const $digits_name: $digits_type = $digits;
        static $dcnumber_name: UnsignedDCNumber = UnsignedDCNumber {
            digits: Cow::Borrowed(&$digits_name),
            separator: ::std::mem::size_of::<$digits_type>(),
        };
    };
}

static_unsigned_dcnumber![ZERO; ZERO_DIGITS: [u8; 1] = [0]];
static_unsigned_dcnumber![ONE; ONE_DIGITS: [u8; 1] = [1]];
static_unsigned_dcnumber![MAX_U64; MAX_U64_DIGITS: [u8; 20] = [1,8,4,4,6, 7,4,4, 0,7,3, 7,0,9, 5,5,1, 6,1,5]];
static_unsigned_dcnumber![MAX_I64; MAX_I64_DIGITS: [u8; 19] = [9,2,2,3,3,7,2,0,3,6,8,5,4,7,7,5,8,0,7]];

impl<'a> UnsignedDCNumber<'a> {

    
    pub fn new<T>(digits: T, last_integer: usize)  -> Self
    where Cow<'a, [u8]>: From<T>
    {   
        UnsignedDCNumber{digits: digits.into(), separator: last_integer}
    }

    pub fn with_integer_digits<T>(digits: T)  -> Self 
    where Cow<'a, [u8]>: From<T>
    {
        let digits: Cow<'a, [u8]> = digits.into();
        let size = digits.len();
        UnsignedDCNumber{digits, separator: size}
        
    }

    #[allow(dead_code)]
    #[inline]
    pub fn integer_magnitude(&self) -> usize {
        self.separator
    }

    #[allow(dead_code)]
    #[inline]
    pub fn fractional_digits(&self) -> usize {
        self.digits.len() - self.separator
    }

    #[inline]
    fn fractional(&self) -> &[u8] {
        self.split().1
    }

    #[inline]
    fn integer(&self) -> &[u8] {
        self.split().0
    }

    #[inline]
    fn split(&self) -> (&[u8], &[u8]) {
        let im = self.integer_magnitude();
        (&self.digits[..im], &self.digits[im..])
    }

    fn blind_to_u64(&self) -> u64 {
        self.integer()
            .iter()
            .cloned()
            .fold(0, |acc, d| acc * 10 + d as u64)
    }

    fn cmp_unsigned<'b>(&self, other: &UnsignedDCNumber<'b>) -> Ordering {
        let self_integer = self.integer_magnitude();
        let other_integer = other.integer_magnitude();
        match self_integer.cmp(&other_integer) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                match self.digits[..self_integer].cmp(&other.digits[..other_integer]) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => {
                        self.digits[self_integer..].cmp(&other.digits[other_integer..])
                    }
                }
            }
        }
    }

    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseDCNumberError> {
        assert_eq!(10, radix);
        let mut first_dot: Option<usize> = None;
        // use vecdeq preferentially
        let no_digits = s.len();
        let mut digits = Vec::with_capacity(no_digits);

        if s.is_empty() {
            return Err(ParseDCNumberError{kind: ParseDCNumberErrorKind::EmptyString});
        }
        let mut bytes = s.bytes();



        match bytes.by_ref().enumerate().find(|(i, d)| { if *d != b'0' { true } else { false }}) {
            None => {
                // if we are here, it means they are all zeros: we did not find any non zero character
                return Ok(ZERO.clone());
            } 
            Some((0,  b'.')) => {
                // TODO: if we do not do this, we will not have a leading 0, which might be desirable
                digits.push(0);
                first_dot = Some(1);
            }
            Some((non_zero_index,  b'.')) => {
                first_dot = Some(non_zero_index);
                // TODO: if we do not do this, we will not have a leading 0, which might be desirable
                digits.push(0);
            }
            Some((_non_zero_index, ch @ b'1'...b'9')) => {
                digits.push(ch - b'0');
            }
            Some((_non_zero_index, _non_zero_byte)) => {
                return Err(ParseDCNumberError{kind: ParseDCNumberErrorKind::InvalidDigit});
            }
        }

        // if we are here, we have one non zero character:
        // * if it was dot, we have marked first_dot and added the zero digit
        // * if it is a valid digit, we have added it among the digits
        // * if they were all 0s or the first non zero was not a digit, it is a parse error and we
        //   would not be here

        for (i, ch) in bytes.enumerate() {
            match ch {
                d @ b'0'...b'9' => digits.push(d - b'0'),
                b'.' => {
                    if let None = first_dot {
                        first_dot = Some(i+1);
                    } else {
                        return Err(ParseDCNumberError {kind: ParseDCNumberErrorKind::RepeatedDot});
                    }
                }
                _ => return Err(ParseDCNumberError {kind: ParseDCNumberErrorKind::InvalidDigit}),
            }
        }

        
        if let Some(..) = first_dot {
            loop {
                match digits.pop()  {
                    Some(0) => {
                        continue;
                    }
                    Some(ch) => {
                        digits.push(ch);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
        }


        let separator = first_dot.unwrap_or(digits.len());
        Ok(UnsignedDCNumber::new(digits, separator))
    }
}

#[test]
fn test_split() {
    assert_eq!(([0 as u8].as_ref(), [].as_ref()), ZERO.split());
    assert_eq!(([1 as u8].as_ref(), [].as_ref()), ONE.split());
    assert_eq!(([1, 2, 3, 4].as_ref(), [3, 2].as_ref()), UnsignedDCNumber::from_str("1234.32").expect("1234.32").split());
    assert_eq!(([1, 2, 3, 4].as_ref(), [3, 2].as_ref()), UnsignedDCNumber::from_str("1234.320").expect("1234.320").split());
}

impl<'a> Default for UnsignedDCNumber<'a> {
    fn default() -> Self {
        ZERO.clone()
    }
}

#[test]
fn test_default() {
    assert_eq!(ZERO, UnsignedDCNumber::default());
}

impl<'a> PartialEq for UnsignedDCNumber<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_unsigned(other) == Ordering::Equal
    }
}

impl<'a> Eq for UnsignedDCNumber<'a> {}

impl<'a> PartialOrd for UnsignedDCNumber<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_unsigned(other))
    }
}

impl<'a> Ord for UnsignedDCNumber<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_unsigned(other)
    }
}

#[test]
fn test_cmp_unsigned() {
    assert_eq!(Ordering::Equal, ZERO.cmp_unsigned(&ZERO));
    assert_eq!(Ordering::Less, ZERO.cmp_unsigned(&ONE));
    assert_eq!(Ordering::Greater, ONE.cmp_unsigned(&ZERO));
    assert_eq!(Ordering::Equal, ONE.cmp_unsigned(&ONE));
}

#[test]
fn test_eq() {
    assert_eq!(ZERO, ZERO);
    assert_eq!(ONE, ONE)
}

#[test]
fn test_partial_order() {
    assert_eq!(Some(Ordering::Less), ZERO.partial_cmp(&ONE));
    assert_eq!(Some(Ordering::Greater), ONE.partial_cmp(&ZERO));
    assert_eq!(Some(Ordering::Equal), ZERO.partial_cmp(&ZERO));
    assert_eq!(Some(Ordering::Less), UnsignedDCNumber::from(213).partial_cmp(&UnsignedDCNumber::from_str("321.12").unwrap()));
}

#[test]
fn test_order() {
    assert!(ZERO < ONE);
}

// TODO add similar to test_partial_order for cmp as well

impl<'a> ToPrimitive for UnsignedDCNumber<'a> {
    fn to_u64(&self) -> Option<u64> {
        if self.fractional().iter().cloned().any(|d| d != 0) {
            return None;
        }

        // various optimizations are possible here
        if self > &MAX_U64 {
            return None;
        }

        Some(self.blind_to_u64())
    }
    fn to_i64(&self) -> Option<i64> {
        if self.fractional().iter().cloned().any(|d| d != 0) {
            return None;
        }

        if self > &MAX_I64 {
            return None;
        }

        Some(self.blind_to_u64() as i64)
    }
}

#[test]
fn test_to_primitive() {
    assert_eq!(0, ZERO.to_u64().expect("u64 zero"));
    assert_eq!(1, ONE.to_u64().expect("u64 one"));
    assert_eq!(::std::u64::MAX, MAX_U64.to_u64().expect("u64 max_u64"));
    assert_eq!(
        ::std::i64::MAX as u64,
        MAX_I64.to_u64().expect("u64 max_i64")
    );
    assert_eq!(
        ::std::i64::MAX as u64 + 1,
        MAX_I64.to_u64().expect("u64 max_i64") + 1
    );

    assert_eq!(0, ZERO.to_i64().expect("i64 zero"));
    assert_eq!(1, ONE.to_i64().expect("i64 one"));
    assert_eq!(None, MAX_U64.to_i64());
    assert_eq!(::std::i64::MAX, MAX_I64.to_i64().expect("i64 max_i64"));
}

// impl <'a> num::Zero for UnsignedDCNumber<'a> {
//     fn zero() -> Self {
//         ZERO.clone()
//     }

//     fn is_zero(&self) -> bool {
//         false
//     }
// }

// we should make this for all integers...

const LOG_10_2: f32 = f32::consts::LN_2 / f32::consts::LN_10;

#[inline]
fn decimal_digits(n: u64) -> u32 {
    match n {
        0 => 0,
        1...9 => 1,
        10...100 => 2,
        100...1000 => 3,
        1000...10000 => 4,
        10000...100000 => 5,
        100000...1000000 => 6,
        1000000...10000000 => 7,
        10000000...100000000 => 8,
        _ => {
            // no casting error until we have number with a number of bytes which would not fit in u32
            let first_on = (8 * ::std::mem::size_of::<u64>() as u32) - n.leading_zeros();
            // this formula relies on first_on being > 1, which clearly is
            (first_on as f32 * LOG_10_2) as u32 + 1
        }
    }
}

#[test]
fn test_decimal_digits() {
    assert_eq!(0, decimal_digits(0));
    assert_eq!(1, decimal_digits(1));
    assert_eq!(2, decimal_digits(22));
    assert_eq!(3, decimal_digits(311));
    assert_eq!(4, decimal_digits(4123));
    assert_eq!(5, decimal_digits(63413));
    assert_eq!(6, decimal_digits(732142));
    assert_eq!(7, decimal_digits(9231763));
    assert_eq!(8, decimal_digits(84985731));
    assert_eq!(9, decimal_digits(223173622));
    assert_eq!(10, decimal_digits(1231736322));
    assert_eq!(19, decimal_digits(::std::i64::MAX as u64));
    assert_eq!(20, decimal_digits(::std::u64::MAX));
}

impl<'a> From<u64> for UnsignedDCNumber<'a> {
    fn from(n: u64) -> Self {
        let n_digits = decimal_digits(n) as usize;
        if n_digits == 0 {
            return UnsignedDCNumber::default();
        }
        let mut digits = Vec::with_capacity(n_digits);

        unsafe {
            let mut m = n;
            for i in 1..n_digits {
                *digits.get_unchecked_mut(n_digits - i) = (m % 10) as u8;
                m /= 10;
            } 
            *digits.get_unchecked_mut(0) = (m % 10) as u8;
            digits.set_len(n_digits);
        }

        UnsignedDCNumber::with_integer_digits(digits)
    }
}

#[test]
fn test_from_u64_zero() {
    let zero = UnsignedDCNumber::from(0);
    assert_eq!(ZERO, zero);
}

#[test]
fn test_from_u64_one() {
    let one = UnsignedDCNumber::from(1);
    assert_eq!(ONE, one);
}

#[test]
fn test_from_u64() {
    let n = UnsignedDCNumber::from(1234567890);
    assert_eq!(
        UnsignedDCNumber::with_integer_digits([1, 2, 3, 4, 5,6 , 7, 8, 9, 0].as_ref()),
        n
    );
}

impl<'a> FromStr for UnsignedDCNumber<'a> {
    type Err = ParseDCNumberError; // they are decimal floating point afterfall

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UnsignedDCNumber::from_str_radix(s, 10)
    }
}

#[test]
fn test_from_str() {
    assert_eq!(ZERO, UnsignedDCNumber::from_str("0").expect("0"));
    assert_eq!(ONE, UnsignedDCNumber::from_str("1").expect("1"));

    assert_eq!(
        Err(ParseDCNumberError{kind: ParseDCNumberErrorKind::EmptyString}),
        UnsignedDCNumber::from_str("")
    );

    assert_eq!(
       UnsignedDCNumber::from_str("1234.32").expect("1234.32"),
       UnsignedDCNumber::new([1, 2, 3, 4, 3, 2].as_ref(), 4)
    );

    assert_eq!(
        UnsignedDCNumber::from(1234),
        UnsignedDCNumber::from_str("1234").expect("1234")
    );

    assert_eq!(
        UnsignedDCNumber::from(1234),
        UnsignedDCNumber::from_str("01234").expect("01234")
    );

    assert_eq!(
       UnsignedDCNumber::from_str("1234.32").expect("1234.32"),
       UnsignedDCNumber::from_str("1234.320").expect("1234.32") 
    );

    assert_eq!(
       UnsignedDCNumber::from_str("1234").expect("01234"),
       UnsignedDCNumber::from_str("1234.0").expect("1234.0") 
    );

    assert_eq!(
       UnsignedDCNumber::from_str("1234").expect("01234"),
       UnsignedDCNumber::from_str("1234.").expect("1234.") 
    );

    assert_eq!(
       UnsignedDCNumber::from_str(".32").expect(".32"),
       UnsignedDCNumber::from_str("0.32").expect(".32") 
    );

    assert_eq!(
       UnsignedDCNumber::from_str(".320").expect(".320"),
       UnsignedDCNumber::from_str("0.32").expect(".32") 
    );

    assert_eq!(
       UnsignedDCNumber::from_str("01234.32").expect("01234.32"),
       UnsignedDCNumber::from_str("1234.32").expect("1234.32") 
    );

    assert_eq!(
       UnsignedDCNumber::from_str("01234.32").expect("01234.32"),
       UnsignedDCNumber::from_str("1234.320").expect("1234.320") 
    );



}



// impl <'a> ToPrimitive for DCNumber<'a> {
//     to_u64(&self) -> Option<u64> {
//         if self.
//     }

// }

// impl <'a> PartialEq for DCNumber {

// }

// impl <'a> Ord for DCNumber<'a> {

// }

// #[test]
// fn zero_test() {
//     // assert_eq!(DCNumber::default(), 0);
// }

// impl <'a> From<u64> for DCNumber<'a> {
//     fn from(n: u64) -> DCNumber<'a> {
//         let n_digits = n_digits!(n);
//         if n_digits == 0 {
//             return DCNumber::default();
//         }
//         let mut digits = Vec::with_capacity(n_digits);
//         let index = digits.len()-1;

//         let mut m = n;
//         while index < n_digits {
//             digits[index] = (m % 10) as u8;
//             m /= 10;
//         }

//         let digits = Vec::new();
//         DCNumber::<'a>{digits: Cow::Owned(digits), separator:0, sign: Sign::Plus}
//     }
// }

// #[test]
// fn test_u64_zero() {
//     let zero = DCNumber::from(0);

// }

// impl <'a> ToPrimitive for DCNumber<'a> {
//     to_u64(&self) -> Option<u64> {
//         if self.
//     }

// }

// impl <'a> PartialEq for DCNumber {

// }

// impl <'a> Ord for DCNumber<'a> {

// }

// #[test]
// fn zero_test() {
//     // assert_eq!(DCNumber::default(), 0);
// }zz// #[derive(Clone, Debug)]
// pub struct DCNumber<'a> {
//     n: UnsignedDCNumber<'a>,
//     sign: Sign,
// }

// impl <'a> From<UnsignedDCNumber<'a>> for DCNumber<'a> {
//     fn from(n: UnsignedDCNumber<'a>) -> Self {
//         DCNumber{n, sign: Sign::Plus}
//     }
// }

// #[test]
// fn test_cmp_unsigned() {
//     assert_eq!(Ordering::Equal, ZERO.cmp_unsigned(&ZERO));
//     assert_eq!(Ordering::Less, ZERO.cmp_unsigned(&ONE));
//     assert_eq!(Ordering::Less, ZERO.cmp_unsigned(&MINUS_ONE));
//     assert_eq!(Ordering::Greater, ONE.cmp_unsigned(&ZERO));
//     assert_eq!(Ordering::Greater, MINUS_ONE.cmp_unsigned(&ZERO));
//     assert_eq!(Ordering::Equal, MINUS_ONE.cmp_unsigned(&ONE));
//     assert_eq!(Ordering::Equal, ONE.cmp_unsigned(&MINUS_ONE));
//     assert_eq!(Ordering::Equal, ONE.cmp_unsigned(&ONE));
//     assert_eq!(Ordering::Equal, MINUS_ONE.cmp_unsigned(&MINUS_ONE));
// }

// static MINUS_ONE: DCNumber = DCNumber{ n: ONE.clone(), sign: Sign::Minus};

// impl <'a> num::Num for DCNumber<'a> {

// }
