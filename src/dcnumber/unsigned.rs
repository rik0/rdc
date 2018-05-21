use super::error::ParseDCNumberError;
use super::traits::FromBytes;
use num::ToPrimitive;
use std::borrow::Cow;
use std::cmp::{max, Ordering};
use std::collections::VecDeque;
use std::f32;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::iter::{self, Iterator};
use std::ops::{Add, Mul, Range};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct UnsignedDCNumber<'a> {
    // TODO: maybe use nibble?
    // digits are in BigEndian
    digits: Cow<'a, [u8]>,
    // also consider having a pool for these numbers for memory locality
    separator: usize,
}

macro_rules! static_unsigned_dcnumber {
    ($dcnumber_name:ident; $digits_name:ident : $digits_type:ty = $digits:expr) => {
        #[allow(dead_code)]
        const $digits_name: $digits_type = $digits;
        #[allow(dead_code)]
        static $dcnumber_name: UnsignedDCNumber = UnsignedDCNumber {
            digits: Cow::Borrowed(&$digits_name),
            separator: ::std::mem::size_of::<$digits_type>(),
        };
    };
}

#[cfg(test)]
macro_rules! udcn {
    ($digits:expr) => {
        UnsignedDCNumber::from_str($digits).expect(stringify!($digits))
    };
}

// TODO: at some point we want to make a real preallocated space for these
static_unsigned_dcnumber![ZERO; ZERO_DIGITS: [u8; 1] = [0]];
static_unsigned_dcnumber![ONE; ONE_DIGITS: [u8; 1] = [1]];
static_unsigned_dcnumber![MAX_U64; MAX_U64_DIGITS: [u8; 20] = [1,8,4,4,6, 7,4,4, 0,7,3, 7,0,9, 5,5,1, 6,1,5]];
static_unsigned_dcnumber![MAX_I64; MAX_I64_DIGITS: [u8; 19] = [9,2,2,3,3,7,2,0,3,6,8,5,4,7,7,5,8,0,7]];

impl<'a> UnsignedDCNumber<'a> {
    pub fn new<T>(digits: T, last_integer: usize) -> Self
        where
            Cow<'a, [u8]>: From<T>,
    {
        UnsignedDCNumber {
            digits: digits.into(),
            separator: last_integer,
        }
    }

    pub fn with_integer_digits<T>(digits: T) -> Self
        where
            Cow<'a, [u8]>: From<T>,
    {
        let digits: Cow<'a, [u8]> = digits.into();
        let size = digits.len();
        UnsignedDCNumber {
            digits,
            separator: size,
        }
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
}

impl<'a> Default for UnsignedDCNumber<'a> {
    fn default() -> Self {
        ZERO.clone()
    }
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

// TODO add similar to test_partial_order for cmp as well

impl<'a> ToPrimitive for UnsignedDCNumber<'a> {
    fn to_i64(&self) -> Option<i64> {
        if self.fractional().iter().cloned().any(|d| d != 0) {
            return None;
        }

        if self > &MAX_I64 {
            return None;
        }

        Some(self.blind_to_u64() as i64)
    }
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
}

impl<'a> Display for UnsignedDCNumber<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use std::fmt::Write;

        for &ch in &self.digits[0..self.separator] {
            f.write_char((ch + b'0') as char)?;
        }
        if self.separator != self.digits.len() {
            f.write_char('.')?;
            for &ch in &self.digits[self.separator..] {
                f.write_char((ch + b'0') as char)?;
            }
        }
        Ok(())
    }
}

impl<'a> Add<u64> for UnsignedDCNumber<'a> {
    type Output = UnsignedDCNumber<'a>;

    fn add(self, other: u64) -> Self::Output {
        // TODO make this more efficient by implementing Add "in place"
        self + UnsignedDCNumber::from(other)
    }
}

macro_rules! lsd {
    ($n:expr) => {
        ($n % 10) as u8
    };
}

impl<'a> Mul<u8> for UnsignedDCNumber<'a> {
    type Output = UnsignedDCNumber<'a>;

    fn mul(self, other: u8) -> Self::Output {
        // optimize 0, 1, 10, 100

        let mut separator = self.separator;
        let mut digits = VecDeque::from(self.digits.into_owned());

        let mut index = 0;
        loop {
            if index >= digits.len() {
                break;
            }

            if digits[index] == 0 {
                index += 1;
                continue;
            }

            // we are really multiplying two u8 so w need an u16 for the result without error
            let mut result = digits[index] as u16 * other as u16;
            {
                // this handles the current digit, we need to overwrite what was there
                digits[index] = lsd!(result);
                result /= 10;

                for index in (index.saturating_sub(3)..index).rev() {
                    digits[index] += lsd![result];
                    result /= 10;

                    // here we handle the carry
                    if digits[index] >= 10 {
                        debug_assert!(digits[index] < 20);
                        result += 1;
                        digits[index] -= 10;
                    }
                }
            }

            // if we had "overflow" for this digit, we should create the right
            while result > 0 {
                digits.push_front(lsd![result]);
                separator += 1;
                index += 1;
                result /= 10;
            }
            index += 1;
        }

        digits
            .iter()
            .enumerate()
            .rposition(|(i, &ch)| ch != 0 && i >= separator)
            .map(|last_non_zero| digits.truncate(last_non_zero + 1))
            .unwrap_or_else(|| digits.truncate(separator));

        UnsignedDCNumber::new(Vec::from(digits), separator)
    }
}

// TODO consiider implementing *= u8; migght be the fastest option here (MulAssign)

impl<'a> Add for UnsignedDCNumber<'a> {
    type Output = UnsignedDCNumber<'a>;

    fn add<'b>(self, other: UnsignedDCNumber<'b>) -> Self {
        // TODO since we consume self, we can possibly see if we can reuse the memory buffer
        // implementing this "in place"
        let self_separator = self.separator;
        let other_separator = other.separator;
        let sum_digits_len = max(self.fractional_digits(), other.fractional_digits())
            + max(self.integer_magnitude(), other.integer_magnitude());
        let mut sum_digits = VecDeque::with_capacity(sum_digits_len);

        let self_fractional_len = self.fractional_digits();
        let other_fractional_len = other.fractional_digits();
        let fractional_tail: Vec<u8>;

        let mut self_digits = self.digits.into_owned();
        let mut other_digits = other.digits.into_owned();

        if self_fractional_len > other_fractional_len {
            let offset = self_digits.len() - (self_fractional_len - other_fractional_len);
            fractional_tail = self_digits.split_off(offset);
        } else {
            let offset = other_digits.len() - (other_fractional_len - self_fractional_len);
            fractional_tail = other_digits.split_off(offset);
        }

        let mut carry = false;
        for (mut lhs, rhs) in self_digits
            .into_iter()
            .rev()
            .zip(other_digits.into_iter().rev())
            {
            // as long as we represent internally as an array of u8, this is cheaper than the
            // alternatives. there's no way to wrap around because lhs and rhs are both < 10.
            // this is unfortunately not enforced. we should have a type for "vector of digits"
            // similarly to how strings are implemented by checking the true nature of the digits.
            let value = lhs + rhs + if carry {
                carry = false;
                1
            } else {
                0
            };
            if value >= 10 {
                debug_assert!(value < 20);
                carry = true;
                sum_digits.push_front(value - 10);
            } else {
                sum_digits.push_front(value)
            }
        }

        if carry {
            sum_digits.push_front(1);
        }

        let separator: usize = max(
            max(self_separator, other_separator) + if carry { 1 } else { 0 },
            1,
        );

        sum_digits.extend(fractional_tail);
        UnsignedDCNumber::new(Vec::from(sum_digits), separator)
    }
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

mod radix_converters {
    use super::{ParseDCNumberError, UnsignedDCNumber};

    pub trait AsciiConverter {
        fn append_digits(
            &self,
            digits: &mut Vec<u8>,
            buffer: &[u8],
        ) -> Result<usize, ParseDCNumberError>;

        fn convert_bytes<'a, 'b>(
            &self,
            bytes: &'a [u8],
        ) -> Result<UnsignedDCNumber<'b>, ParseDCNumberError> {
            if bytes.is_empty() {
                return Err(ParseDCNumberError::EmptyString);
            }

            let no_digits = bytes.len();
            let mut digits = Vec::<u8>::with_capacity(no_digits);
            let (integer_part, fractional_part) = split_fractional(bytes);

            let separator = integer_part
                .iter()
                .position(|&ch| ch != b'0')
                .map(|separator| self.append_digits(&mut digits, &integer_part[separator..]))
                .unwrap_or_else(|| {
                    digits.push(0);
                    Ok(1)
                })?;
            let _fractional_items = fractional_part
                .iter()
                .skip(1)  // this is the dot
                .rposition(|&ch| ch != b'0')
                .map(|last_non_zero| self.append_digits(&mut digits, &fractional_part[1..last_non_zero + 2]))
                .unwrap_or(Ok(0))?;
            Ok(UnsignedDCNumber::new(digits, separator))
        }
    }

    #[inline]
    fn split_fractional(bytes: &[u8]) -> (&[u8], &[u8]) {
        let dot = bytes.iter().position(|&ch| ch == b'.');
        match dot {
            None => (bytes, &[][..]),
            Some(dot) => bytes.split_at(dot),
        }
    }

    #[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
    pub struct DecAsciiConverter {}

    impl DecAsciiConverter {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl AsciiConverter for DecAsciiConverter {
        #[inline]
        fn append_digits(
            &self,
            digits: &mut Vec<u8>,
            buffer: &[u8],
        ) -> Result<usize, ParseDCNumberError> {
            let mut counter = 0;
            for &ch in buffer {
                match ch {
                    ch @ b'0'...b'9' => {
                        digits.push(ch - b'0');
                        counter += 1;
                    }
                    b'.' => return Err(ParseDCNumberError::RepeatedDot),
                    _other => return Err(ParseDCNumberError::InvalidDigit),
                };
            }
            Ok(counter)
        }
    }

    #[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
    pub struct Small {
        radix: u8,
    }

    impl Small {
        pub fn new(radix: u8) -> Self {
            assert!(radix <= 10 && radix >= 2);
            Self { radix }
        }

        #[inline]
        fn radix_coversion(&self, digit: u8) -> Result<u8, ParseDCNumberError> {
            match digit {
                ch @ 0...9 if ch - b'0' <= self.radix => Ok(ch - b'0'),
                b'.' => Err(ParseDCNumberError::RepeatedDot),
                _other => Err(ParseDCNumberError::InvalidRadix),
            }
        }

        fn build_partial<'a, 'b>(
            &self,
            _initial: &mut UnsignedDCNumber<'a>,
            _bytes: &'b [u8],
            _start_magnitude: u32,
        ) {
            // we are not checking that the digit is valid
            //            bytes.iter()
            //                .rev()
            //                .map(|&ch| {
            //                    let number_value = self.radix_coversion(ch)?;
            //                    let unsigned_dcnumber = UnsignedDCNumber::from(number_value as u64); // we can optimize creating implementation for u8
            //                    // here we should multiply the value
            //                }) // here we have a bunch of small numbers that will build up the main number
            unimplemented!();
        }
    }

    impl AsciiConverter for Small {
        #[inline]
        fn append_digits(
            &self,
            digits: &mut Vec<u8>,
            buffer: &[u8],
        ) -> Result<usize, ParseDCNumberError> {
            let mut counter = 0;
            let range_top = b'0' + self.radix;
            for &ch in buffer {
                match ch {
                    ch @ b'0'...b'9' if ch <= range_top => {
                        digits.push(ch - b'0');
                        counter += 1;
                    }
                    b'.' => return Err(ParseDCNumberError::RepeatedDot),
                    _other => return Err(ParseDCNumberError::InvalidDigit),
                };
            }
            Ok(counter)
        }

        fn convert_bytes<'a, 'b>(
            &self,
            bytes: &'a [u8],
        ) -> Result<UnsignedDCNumber<'b>, ParseDCNumberError> {
            if bytes.is_empty() {
                return Err(ParseDCNumberError::EmptyString);
            }

            let no_digits = bytes.len();
            let mut digits = Vec::<u8>::with_capacity(no_digits);
            let (integer_part, fractional_part) = split_fractional(bytes);

            let separator = integer_part
                .iter()
                .position(|&ch| ch != b'0')
                .map(|separator| self.append_digits(&mut digits, &integer_part[separator..]))
                .unwrap_or_else(|| {
                    digits.push(0);
                    Ok(1)
                })?;
            let _fractional_items = fractional_part
                .iter()
                .skip(1)  // this is the dot
                .rposition(|&ch| ch != b'0')
                .map(|last_non_zero| self.append_digits(&mut digits, &fractional_part[1..last_non_zero + 2]))
                .unwrap_or(Ok(0))?;
            Ok(UnsignedDCNumber::new(digits, separator))
        }
    }

    #[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
    pub struct Large {
        radix: u8,
    }

    impl Large {
        pub fn new(radix: u8) -> Self {
            assert!(radix >= 11 && radix <= 16);
            Self { radix }
        }
    }

    impl AsciiConverter for Large {
        #[inline]
        fn append_digits(
            &self,
            digits: &mut Vec<u8>,
            buffer: &[u8],
        ) -> Result<usize, ParseDCNumberError> {
            let mut counter = 0;
            let decimal_range_top = b'0' + self.radix;
            let extended_range_top = b'A' + (self.radix - 11);

            let mut carry: u8 = 0;
            for &ch in buffer.iter().rev() {
                match ch {
                    ch @ b'0'...b'9' if ch <= decimal_range_top => {
                        let mut digital_value = ch - b'0' + carry;
                        carry = 0;
                        if digital_value >= 10 {
                            digital_value -= 10;
                            carry = 1;
                        }
                        digits.push(digital_value);
                        counter += 1;
                    }
                    ch @ b'A'...b'F' if ch <= extended_range_top => {
                        let mut digit_value = ch - b'A' + 10 + carry;
                        carry = 0;
                        if digit_value >= 10 {
                            digit_value -= 10;
                            carry = 1;
                        }
                        digits.push(digit_value);
                        counter += 1;
                    }
                    b'.' => return Err(ParseDCNumberError::RepeatedDot),
                    _other => return Err(ParseDCNumberError::InvalidDigit),
                };
            }
            if carry > 0 {
                digits.push(1);
                counter += 1;
            }
            digits.reverse();
            Ok(counter)
        }
    }
}

impl<'a> FromBytes for UnsignedDCNumber<'a> {
    type Err = ParseDCNumberError;

    fn from_bytes_radix(bytes: &[u8], radix: u32) -> Result<Self, ParseDCNumberError> {
        use self::radix_converters::AsciiConverter;

        // TODO Small is now untested.
        // TODO we have too many implementations that work on decimal
        // - from_bytes directly
        // - DecAsciiConverter
        // - Small (with radix 10)
        match radix {
            2...9 => radix_converters::Small::new(radix as u8).convert_bytes(bytes),
            10 => radix_converters::DecAsciiConverter::new().convert_bytes(bytes),
            11...16 => radix_converters::Large::new(radix as u8).convert_bytes(bytes),
            _ => Err(ParseDCNumberError::InvalidRadix),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseDCNumberError> {
        if bytes.is_empty() {
            return Err(ParseDCNumberError::EmptyString);
        }

        let mut first_dot: Option<usize> = None;
        // use vecdeq preferentially
        let no_digits = bytes.len();
        let mut digits = Vec::with_capacity(no_digits);

        let mut zero_streak: Option<Range<usize>> = None;
        let mut seen_non_zero: bool = false;
        let mut skipped_leading_zeros: usize = 0;

        for (pos, &ch) in bytes.iter().enumerate() {
            match ch {
                b'0' => {
                    zero_streak = match zero_streak {
                        None => Some(pos..pos + 1),
                        Some(Range { start, end }) => Some(start..end + 1),
                    };
                }
                ch @ b'1'...b'9' => {
                    if let Some(Range { start, end }) = zero_streak {
                        // we should do this after the dot in non terminal position
                        // and before the dot, but only if we have already seen something non zero
                        debug_assert!(bytes[start..end].iter().all(|&ch| ch == b'0'));
                        if seen_non_zero || first_dot.is_some() {
                            digits.extend(iter::repeat(0).take(end - start));
                        } else if first_dot.is_none() {
                            skipped_leading_zeros += end - start;
                        }
                        zero_streak = None;
                    }
                    digits.push(ch - b'0');
                    seen_non_zero = true;
                }
                b'.' => {
                    if let Some(_) = first_dot {
                        return Err(ParseDCNumberError::RepeatedDot);
                    }
                    if let Some(Range { start, end }) = zero_streak {
                        // this is a number w
                        debug_assert!(bytes[start..end].iter().all(|&ch| ch == b'0'));
                        if seen_non_zero {
                            digits.extend(iter::repeat(0).take(end - start));
                        } else {
                            digits.push(0);
                            skipped_leading_zeros += (end - start) - 1;
                        }
                        zero_streak = None;
                        first_dot = Some(pos);
                    } else if !seen_non_zero {
                        digits.push(0);
                        first_dot = Some(pos + 1);
                    } else {
                        first_dot = Some(pos);
                    }
                    seen_non_zero = true;
                }
                _ => {
                    return Err(ParseDCNumberError::InvalidDigit);
                }
            }
        }

        // if we are not after a dot, we must consider the zero streak here
        if let (Some(Range { start, end }), None) = (zero_streak, first_dot) {
            digits.extend(iter::repeat(0).take(end - start));
        }

        let separator = first_dot
            .map(|len| len - skipped_leading_zeros)
            .unwrap_or(digits.len());
        Ok(UnsignedDCNumber::new(digits, separator))
    }

    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ParseDCNumberError> {
        UnsignedDCNumber::from_bytes_radix(s.as_ref(), radix)
    }
}

impl<'a> FromStr for UnsignedDCNumber<'a> {
    type Err = ParseDCNumberError;

    fn from_str(s: &str) -> Result<Self, ParseDCNumberError> {
        FromBytes::from_bytes(s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(ZERO, UnsignedDCNumber::default());
    }

    #[test]
    fn test_split() {
        assert_eq!(([0 as u8].as_ref(), [].as_ref()), ZERO.split());
        assert_eq!(([1 as u8].as_ref(), [].as_ref()), ONE.split());
        assert_eq!(
            ([1, 2, 3, 4].as_ref(), [3, 2].as_ref()),
            udcn!("1234.32").split()
        );
        assert_eq!(
            ([1, 2, 3, 4].as_ref(), [3, 2].as_ref()),
            UnsignedDCNumber::from_str("1234.320")
                .expect("1234.320")
                .split()
        );
    }

    macro_rules! test_eq {
        ($test_name:ident : $expected_digits:tt = $digits:tt) => {
            mod $test_name {
                use super::*;
                #[test]
                fn eq() {
                    // the purpose of this test is to test equality of things expected equal
                    assert_eq!(
                        udcn![stringify!($expected_digits)],
                        udcn![stringify!($digits)]
                    );
                }

                // these tests keep in sync the various implementations
                #[test]
                fn str_radix_bytes_radix() {
                    assert_eq!(
                        UnsignedDCNumber::from_str_radix(stringify!($digits).as_ref(), 10)
                            .expect(stringify!($digits)),
                        UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), 10)
                            .expect(stringify!($digits)),
                    );
                }

                #[test]
                fn str_bytes() {
                    assert_eq!(
                        UnsignedDCNumber::from_str(stringify!($digits).as_ref())
                            .expect(stringify!($digits)),
                        UnsignedDCNumber::from_bytes(stringify!($digits).as_ref())
                            .expect(stringify!($digits)),
                    );
                }

                #[test]
                fn str_bytes_radix() {
                    assert_eq!(
                        UnsignedDCNumber::from_str(stringify!($digits).as_ref())
                            .expect(stringify!($digits)),
                        UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), 10)
                            .expect(stringify!($digits)),
                    );
                }
            }
        };
    }

    // TODO: fix me
    //    #[test]
    //    fn test_equal_not_normalized() {
    //        assert_eq!(
    //            UnsignedDCNumber::new([0, 3, 2].as_ref(), 1),
    //            UnsignedDCNumber::new([3, 2].as_ref(), 0),
    //        );
    //    }

    // TODO write proper tests for cmp with macro

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
        assert_eq!(
            Some(Ordering::Less),
            UnsignedDCNumber::from(213).partial_cmp(&UnsignedDCNumber::from_str("321.12").unwrap())
        );
    }

    #[test]
    fn test_order() {
        assert!(ZERO < ONE);
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
            None,
            UnsignedDCNumber::from_str("10.1").expect("10.1").to_u64()
        );
        assert_eq!(None, UnsignedDCNumber::from_str("6125216521678251786215186528167125821752187528175218721582715125214512421532154211624217421765421").expect("huge").to_u64());

        assert_eq!(
            ::std::i64::MAX as u64 + 1,
            MAX_I64.to_u64().expect("u64 max_i64") + 1
        );

        assert_eq!(0, ZERO.to_i64().expect("i64 zero"));
        assert_eq!(1, ONE.to_i64().expect("i64 one"));
        assert_eq!(None, MAX_U64.to_i64());
        assert_eq!(::std::i64::MAX, MAX_I64.to_i64().expect("i64 max_i64"));
        assert_eq!(
            None,
            UnsignedDCNumber::from_str("10.1").expect("10.1").to_i64()
        );
    }

    macro_rules! test_display {
        ($test_name:ident : $digits:tt) => {
            #[test]
            fn $test_name() {
                use std::io::Write;
                let mut out = Vec::new();
                let n = UnsignedDCNumber::from_str($digits).expect($digits);
                let _ = write!(out, "{}", n).expect("write");

                assert_eq!(
                    $digits.to_string(),
                    String::from_utf8(out).expect("utf8 issue")
                )
            }
        };
    }

    #[test]
    fn test_display() {
        use std::io::Write;
        let digits = "0";
        let mut out = Vec::new();
        let n = UnsignedDCNumber::from_str(digits).expect(digits);
        let _ = write!(out, "{}", n).expect("write");

        assert_eq!(
            digits.to_string(),
            String::from_utf8(out).expect("utf8 issue")
        )
    }

    test_display![display_zero: "0"];
    test_display![display_one: "1"];
    test_display![display_one_dot_one: "1.1"];
    test_display![display_1dot1: "1.1"];
    test_display![display_10dot1: "10.1"];
    test_display![display_0dot9: "0.9"];
    test_display![display_0dot01: "0.01"];
    test_display![display_1740: "1740"];
    test_display![display_1000dot3: "1000.3"];

    macro_rules! test_binop {
        ($test_name:ident : $expected:tt = $lhs:tt $op:tt $rhs:tt) => {
            #[test]
            fn $test_name() {
                assert_eq!(
                                udcn![stringify!($expected)],
                                udcn![stringify!($lhs)] $op udcn![stringify!($rhs)],
                            );
            }
        };
        (u8 $test_name:ident : $expected:tt = $lhs:tt $op:tt $rhs:expr) => {
            #[test]
            fn $test_name() {
                assert_eq!(
                                udcn![stringify!($expected)],
                                udcn![stringify!($lhs)] $op $rhs,
                            );
            }
        };
    }

    test_binop![test_add_zero: 0 = 0 + 0];
    test_binop![test_add_unit: 1 = 1 + 0];
    test_binop![test_add_unit2: 1 = 0 + 1];
    test_binop![test_integers: 1026 = 520 + 506];
    test_binop![test_add_frac: 20.2 = 10.1 + 10.1];
    test_binop![test_add_f:10143.043 = 7221.123 + 2921.92];

    mod mul {
        use super::*;

        test_binop![u8 t1: 10 = 1 * 10];
        test_binop![u8 t0: 0 = 0 * 10];
        test_binop![u8 t10: 100 = 10 * 10];
        test_binop![u8 t10dot1: 101 = 10.1 * 10];
        test_binop![u8 t0dot1: 1 = 0.1 * 10];
        test_binop![u8 t1_2: 10 = 1 * 10];
        test_binop![u8 t0_2: 0 = 0 * 2];
        test_binop![u8 t10_2: 20 = 10 * 2];
        test_binop![u8 t10dot1_2: 20.2 = 10.1 * 2];
        test_binop![u8 t0dot1_2: 0.2 = 0.1 * 2];
        test_binop![u8 t19_99: 1881 = 19 * 99];
        test_binop![u8 t109_99: 10791 = 109 * 99];
        test_binop![u8 t109dot0_99: 10791 = 109.0 * 99];
        test_binop![u8 t10dot9_99: 1079.1 = 10.9 * 99];

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
            UnsignedDCNumber::with_integer_digits([1, 2, 3, 4, 5, 6, 7, 8, 9, 0].as_ref()),
            n
        );
    }

    macro_rules! test_from_str {
        ($test_name:ident : $error_id:tt <- $digits:tt) => {
            mod $test_name {
                use super::*;

                #[test]
                fn from_str() {
                    assert_eq!(
                        Err(ParseDCNumberError::$error_id),
                        UnsignedDCNumber::from_str($digits)
                    );
                }

                #[test]
                fn from_bytes() {
                    assert_eq!(
                        Err(ParseDCNumberError::$error_id),
                        UnsignedDCNumber::from_bytes($digits.as_ref())
                    );
                }

                #[test]
                fn from_str_radix() {
                    assert_eq!(
                        Err(ParseDCNumberError::$error_id),
                        UnsignedDCNumber::from_str_radix($digits, 10)
                    );
                }

                #[test]
                fn from_bytes_radix() {
                    assert_eq!(
                        Err(ParseDCNumberError::$error_id),
                        UnsignedDCNumber::from_bytes_radix($digits.as_ref(), 10)
                    );
                }
            }
        };

        ($test_name:ident : $expected:expr; $digits:tt) => {
            mod $test_name {
                use super::*;

                #[test]
                fn ucdn() {
                    assert_eq!($expected, udcn!(stringify!($digits)));
                }

                #[test]
                fn from_bytes() {
                    assert_eq!(
                        $expected,
                        UnsignedDCNumber::from_bytes(stringify!($digits).as_ref())
                            .expect(stringify!($digits))
                    );
                }

                #[test]
                fn from_bytes_radix() {
                    assert_eq!(
                        $expected,
                        UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), 10)
                            .expect(stringify!($digits))
                    );
                }

                #[test]
                fn from_str() {
                    assert_eq!(
                        $expected,
                        UnsignedDCNumber::from_str(stringify!($digits).as_ref())
                            .expect(stringify!($digits))
                    );
                }

                #[test]
                fn from_str_radix() {
                    assert_eq!(
                        $expected,
                        UnsignedDCNumber::from_str_radix(stringify!($digits).as_ref(), 10)
                            .expect(stringify!($digits))
                    );
                }

            }
        };
    }

    macro_rules! bench_from_str {
        ($bench_name:ident : $digits:expr) => {
            #[cfg(all(feature = "nightly", test))]
            mod $bench_name {
                use super::*;
                use test::Bencher;

                #[bench]
                fn test_udcn(b: &mut Bencher) {
                    b.iter(|| {
                        udcn![$digits];
                    });
                }

                #[bench]
                fn test_from_bytes(b: &mut Bencher) {
                    b.iter(|| {
                        UnsignedDCNumber::from_bytes($digits.as_ref()).expect(stringify!($digits))
                    });
                }

                #[bench]
                fn test_from_bytes_radix_10(b: &mut Bencher) {
                    b.iter(|| {
                        UnsignedDCNumber::from_bytes_radix($digits.as_ref(), 10)
                            .expect(stringify!($digits))
                    });
                }

                #[bench]
                fn test_from_str(b: &mut Bencher) {
                    b.iter(|| {
                        UnsignedDCNumber::from_str($digits.as_ref()).expect(stringify!($digits))
                    });
                }

                #[bench]
                fn test_from_str_radix_10(b: &mut Bencher) {
                    b.iter(|| {
                        UnsignedDCNumber::from_str_radix($digits.as_ref(), 10)
                            .expect(stringify!($digits))
                    });
                }
            }
        };
    }

    test_from_str![test_from_str_zero: ZERO ; 0];
    test_from_str![test_from_str_one:  ONE ; 1];
    test_from_str![test_from_str_byte_spec: UnsignedDCNumber::new([1, 1].as_ref(), 1) ; 1.1];
    test_from_str![test_from_str_0dot9: UnsignedDCNumber::new([0, 9].as_ref(), 1) ; 0.9];
    test_from_str![test_from_str_1000dot3: UnsignedDCNumber::new([1, 0, 0, 0, 3].as_ref(), 4) ; 1000.3];
    test_from_str![test_from_str_0dot01: UnsignedDCNumber::new([0, 0, 1].as_ref(), 1) ; 0.01];
    test_from_str![test_from_str_from_int: UnsignedDCNumber::from(1234) ; 1234 ];
    test_from_str![test_from_str_from_int_leading0: UnsignedDCNumber::from(1234) ; 01234];
    test_from_str![test_from_str_empty : EmptyString <- ""];
    test_from_str![test_from_str_a : InvalidDigit <- "a"];
    test_from_str![test_from_str_1a : InvalidDigit <- "1a]"];
    test_from_str![test_from_str_0a : InvalidDigit <- "0a"];
    test_from_str![test_from_str_dota : InvalidDigit <- ".a"];
    test_from_str![test_from_str_0dotdot0: RepeatedDot <- "0..0"];
    test_eq![test_from_tail0 : 1234.32 = 1234.320 ];
    test_eq![test_from_taildot0 : 1234 = 1234.0 ];
    test_eq![test_from_ident : 1234 = 1234.];
    test_eq![test_from_leading0_f : 01234.32 = 1234.32 ];
    test_eq![test_from_leading_tailing_0f : 01234.32 = 1234.320 ];
    test_eq![eq_zero: 0 = 0];
    test_eq![eq_one: 1 = 1];
    test_eq![eq_one_dot_one: 1.1 = 1.1];
    test_eq![eq_0dot9: 0.9 = 0.9];
    test_eq![eq_0dot01: 0.01 = 0.01];
    test_eq![eq_1740: 1740 = 1740];
    test_eq![eq_1000dot3: 1000.3 = 1000.3];

    #[test]
    fn test_from_str_dot32() {
        assert_eq!(
            UnsignedDCNumber::from_str(".32").expect(".32"),
            UnsignedDCNumber::from_str("0.32").expect("0.32")
        );
    }

    #[test]
    fn test_from_str_dot320() {
        assert_eq!(
            UnsignedDCNumber::from_str(".320").expect(".320"),
            UnsignedDCNumber::from_str("0.32").expect("0.32")
        );
    }

    // write test for from_bytes with various bases

    macro_rules! from_bytes_radix {
        ($test_name:ident : $decimal_digits:tt = $digits:tt : $radix:expr) => {
            #[test]
            fn $test_name() {
                assert_eq!(
                    UnsignedDCNumber::from_bytes(stringify!($decimal_digits).as_ref())
                        .expect(stringify!($decimal_digits)),
                    UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), $radix)
                        .expect(stringify!($digits)),
                );
            }
        };
    }

    // TODO reenable when we are done with radix conversions
    //    from_bytes_radix![first_hex: 10 = A: 16];
    //    from_bytes_radix![b2_10: 2 = 10: 2];
    //    from_bytes_radix![b3_10: 3 = 10: 3];
    //    from_bytes_radix![b4_10: 4 = 10: 4];
    //    from_bytes_radix![b5_10: 5 = 10: 5];
    //    from_bytes_radix![b6_10: 6 = 10: 6];
    //    from_bytes_radix![b7_10: 7 = 10: 7];
    //    from_bytes_radix![b8_10: 8 = 10: 8];
    //    from_bytes_radix![b9_10: 9 = 10: 9];

    bench_from_str![short_int: "3"];
    bench_from_str![mid_int: "17235428"];
    bench_from_str![long_int: "172354283422734622371431236441234351267438543781453193415694871634731457681354784531"];
    bench_from_str![longer_int: "17235428342273462237143123644123435126743854378145319341569487000000000000163473145768135478453123187356412946123041213310238698752341280000000000000000000000"];

}
