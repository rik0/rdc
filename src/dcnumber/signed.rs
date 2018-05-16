
// #[derive(Copy, PartialEq, Debug, Clone)]
// enum Sign {
//     Plus,
//     Minus
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
