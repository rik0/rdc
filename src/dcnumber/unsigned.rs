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

use super::util::{CarryingIterator, carrying};


use super::error::ParseDCNumberError;
use super::traits::FromBytes;

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

static_unsigned_dcnumber![ZERO; ZERO_DIGITS: [u8; 1] = [0]];
static_unsigned_dcnumber![ONE; ONE_DIGITS: [u8; 1] = [1]];
static_unsigned_dcnumber![MAX_U64; MAX_U64_DIGITS: [u8; 20] = [1,8,4,4,6, 7,4,4, 0,7,3, 7,0,9, 5,5,1, 6,1,5]];
static_unsigned_dcnumber![MAX_I64; MAX_I64_DIGITS: [u8; 19] = [9,2,2,3,3,7,2,0,3,6,8,5,4,7,7,5,8,0,7]];

mod small_ints {
    use super::*;

    // 0 - 9
    static_unsigned_dcnumber![N0; N0_DIGITS: [u8; 1] = [0]];
    static_unsigned_dcnumber![N1; N1_DIGITS: [u8; 1] = [1]];
    static_unsigned_dcnumber![N2; N2_DIGITS: [u8; 1] = [2]];
    static_unsigned_dcnumber![N3; N3_DIGITS: [u8; 1] = [3]];
    static_unsigned_dcnumber![N4; N4_DIGITS: [u8; 1] = [4]];
    static_unsigned_dcnumber![N5; N5_DIGITS: [u8; 1] = [5]];
    static_unsigned_dcnumber![N6; N6_DIGITS: [u8; 1] = [6]];
    static_unsigned_dcnumber![N7; N7_DIGITS: [u8; 1] = [7]];
    static_unsigned_dcnumber![N8; N8_DIGITS: [u8; 1] = [8]];
    static_unsigned_dcnumber![N9; N9_DIGITS: [u8; 1] = [9]];
    // 10 - 19
    static_unsigned_dcnumber![N10; N10_DIGITS: [u8; 2] = [1, 0]];
    static_unsigned_dcnumber![N11; N11_DIGITS: [u8; 2] = [1, 1]];
    static_unsigned_dcnumber![N12; N12_DIGITS: [u8; 2] = [1, 2]];
    static_unsigned_dcnumber![N13; N13_DIGITS: [u8; 2] = [1, 3]];
    static_unsigned_dcnumber![N14; N14_DIGITS: [u8; 2] = [1, 4]];
    static_unsigned_dcnumber![N15; N15_DIGITS: [u8; 2] = [1, 5]];
    static_unsigned_dcnumber![N16; N16_DIGITS: [u8; 2] = [1, 6]];
    static_unsigned_dcnumber![N17; N17_DIGITS: [u8; 2] = [1, 7]];
    static_unsigned_dcnumber![N18; N18_DIGITS: [u8; 2] = [1, 8]];
    static_unsigned_dcnumber![N19; N19_DIGITS: [u8; 2] = [1, 9]];
    // 20 - 29
    static_unsigned_dcnumber![N20; N20_DIGITS: [u8; 2] = [2, 0]];
    static_unsigned_dcnumber![N21; N21_DIGITS: [u8; 2] = [2, 1]];
    static_unsigned_dcnumber![N22; N22_DIGITS: [u8; 2] = [2, 2]];
    static_unsigned_dcnumber![N23; N23_DIGITS: [u8; 2] = [2, 3]];
    static_unsigned_dcnumber![N24; N24_DIGITS: [u8; 2] = [2, 4]];
    static_unsigned_dcnumber![N25; N25_DIGITS: [u8; 2] = [2, 5]];
    static_unsigned_dcnumber![N26; N26_DIGITS: [u8; 2] = [2, 6]];
    static_unsigned_dcnumber![N27; N27_DIGITS: [u8; 2] = [2, 7]];
    static_unsigned_dcnumber![N28; N28_DIGITS: [u8; 2] = [2, 8]];
    static_unsigned_dcnumber![N29; N29_DIGITS: [u8; 2] = [2, 9]];
    // 30 - 39
    static_unsigned_dcnumber![N30; N30_DIGITS: [u8; 2] = [3, 0]];
    static_unsigned_dcnumber![N31; N31_DIGITS: [u8; 2] = [3, 1]];
    static_unsigned_dcnumber![N32; N32_DIGITS: [u8; 2] = [3, 2]];
    static_unsigned_dcnumber![N33; N33_DIGITS: [u8; 2] = [3, 3]];
    static_unsigned_dcnumber![N34; N34_DIGITS: [u8; 2] = [3, 4]];
    static_unsigned_dcnumber![N35; N35_DIGITS: [u8; 2] = [3, 5]];
    static_unsigned_dcnumber![N36; N36_DIGITS: [u8; 2] = [3, 6]];
    static_unsigned_dcnumber![N37; N37_DIGITS: [u8; 2] = [3, 7]];
    static_unsigned_dcnumber![N38; N38_DIGITS: [u8; 2] = [3, 8]];
    static_unsigned_dcnumber![N39; N39_DIGITS: [u8; 2] = [3, 9]];
    // 40 - 49
    static_unsigned_dcnumber![N40; N40_DIGITS: [u8; 2] = [4, 0]];
    static_unsigned_dcnumber![N41; N41_DIGITS: [u8; 2] = [4, 1]];
    static_unsigned_dcnumber![N42; N42_DIGITS: [u8; 2] = [4, 2]];
    static_unsigned_dcnumber![N43; N43_DIGITS: [u8; 2] = [4, 3]];
    static_unsigned_dcnumber![N44; N44_DIGITS: [u8; 2] = [4, 4]];
    static_unsigned_dcnumber![N45; N45_DIGITS: [u8; 2] = [4, 5]];
    static_unsigned_dcnumber![N46; N46_DIGITS: [u8; 2] = [4, 6]];
    static_unsigned_dcnumber![N47; N47_DIGITS: [u8; 2] = [4, 7]];
    static_unsigned_dcnumber![N48; N48_DIGITS: [u8; 2] = [4, 8]];
    static_unsigned_dcnumber![N49; N49_DIGITS: [u8; 2] = [4, 9]];
    // 50 - 59
    static_unsigned_dcnumber![N50; N50_DIGITS: [u8; 2] = [5, 0]];
    static_unsigned_dcnumber![N51; N51_DIGITS: [u8; 2] = [5, 1]];
    static_unsigned_dcnumber![N52; N52_DIGITS: [u8; 2] = [5, 2]];
    static_unsigned_dcnumber![N53; N53_DIGITS: [u8; 2] = [5, 3]];
    static_unsigned_dcnumber![N54; N54_DIGITS: [u8; 2] = [5, 4]];
    static_unsigned_dcnumber![N55; N55_DIGITS: [u8; 2] = [5, 5]];
    static_unsigned_dcnumber![N56; N56_DIGITS: [u8; 2] = [5, 6]];
    static_unsigned_dcnumber![N57; N57_DIGITS: [u8; 2] = [5, 7]];
    static_unsigned_dcnumber![N58; N58_DIGITS: [u8; 2] = [5, 8]];
    static_unsigned_dcnumber![N59; N59_DIGITS: [u8; 2] = [5, 9]];
    // 60 - 69
    static_unsigned_dcnumber![N60; N60_DIGITS: [u8; 2] = [6, 0]];
    static_unsigned_dcnumber![N61; N61_DIGITS: [u8; 2] = [6, 1]];
    static_unsigned_dcnumber![N62; N62_DIGITS: [u8; 2] = [6, 2]];
    static_unsigned_dcnumber![N63; N63_DIGITS: [u8; 2] = [6, 3]];
    static_unsigned_dcnumber![N64; N64_DIGITS: [u8; 2] = [6, 4]];
    static_unsigned_dcnumber![N65; N65_DIGITS: [u8; 2] = [6, 5]];
    static_unsigned_dcnumber![N66; N66_DIGITS: [u8; 2] = [6, 6]];
    static_unsigned_dcnumber![N67; N67_DIGITS: [u8; 2] = [6, 7]];
    static_unsigned_dcnumber![N68; N68_DIGITS: [u8; 2] = [6, 8]];
    static_unsigned_dcnumber![N69; N69_DIGITS: [u8; 2] = [6, 9]];
    // 70 - 79
    static_unsigned_dcnumber![N70; N70_DIGITS: [u8; 2] = [7, 0]];
    static_unsigned_dcnumber![N71; N71_DIGITS: [u8; 2] = [7, 1]];
    static_unsigned_dcnumber![N72; N72_DIGITS: [u8; 2] = [7, 2]];
    static_unsigned_dcnumber![N73; N73_DIGITS: [u8; 2] = [7, 3]];
    static_unsigned_dcnumber![N74; N74_DIGITS: [u8; 2] = [7, 4]];
    static_unsigned_dcnumber![N75; N75_DIGITS: [u8; 2] = [7, 5]];
    static_unsigned_dcnumber![N76; N76_DIGITS: [u8; 2] = [7, 6]];
    static_unsigned_dcnumber![N77; N77_DIGITS: [u8; 2] = [7, 7]];
    static_unsigned_dcnumber![N78; N78_DIGITS: [u8; 2] = [7, 8]];
    static_unsigned_dcnumber![N79; N79_DIGITS: [u8; 2] = [7, 9]];
    // 80 - 89
    static_unsigned_dcnumber![N80; N80_DIGITS: [u8; 2] = [8, 0]];
    static_unsigned_dcnumber![N81; N81_DIGITS: [u8; 2] = [8, 1]];
    static_unsigned_dcnumber![N82; N82_DIGITS: [u8; 2] = [8, 2]];
    static_unsigned_dcnumber![N83; N83_DIGITS: [u8; 2] = [8, 3]];
    static_unsigned_dcnumber![N84; N84_DIGITS: [u8; 2] = [8, 4]];
    static_unsigned_dcnumber![N85; N85_DIGITS: [u8; 2] = [8, 5]];
    static_unsigned_dcnumber![N86; N86_DIGITS: [u8; 2] = [8, 6]];
    static_unsigned_dcnumber![N87; N87_DIGITS: [u8; 2] = [8, 7]];
    static_unsigned_dcnumber![N88; N88_DIGITS: [u8; 2] = [8, 8]];
    static_unsigned_dcnumber![N89; N89_DIGITS: [u8; 2] = [8, 9]];
    // 90 - 99
    static_unsigned_dcnumber![N90; N90_DIGITS: [u8; 2] = [9, 0]];
    static_unsigned_dcnumber![N91; N91_DIGITS: [u8; 2] = [9, 1]];
    static_unsigned_dcnumber![N92; N92_DIGITS: [u8; 2] = [9, 2]];
    static_unsigned_dcnumber![N93; N93_DIGITS: [u8; 2] = [9, 3]];
    static_unsigned_dcnumber![N94; N94_DIGITS: [u8; 2] = [9, 4]];
    static_unsigned_dcnumber![N95; N95_DIGITS: [u8; 2] = [9, 5]];
    static_unsigned_dcnumber![N96; N96_DIGITS: [u8; 2] = [9, 6]];
    static_unsigned_dcnumber![N97; N97_DIGITS: [u8; 2] = [9, 7]];
    static_unsigned_dcnumber![N98; N98_DIGITS: [u8; 2] = [9, 8]];
    static_unsigned_dcnumber![N99; N99_DIGITS: [u8; 2] = [9, 9]];
    // 100 - 109
    static_unsigned_dcnumber![N100; N100_DIGITS: [u8; 3] = [1, 0, 0]];
    static_unsigned_dcnumber![N101; N101_DIGITS: [u8; 3] = [1, 0, 1]];
    static_unsigned_dcnumber![N102; N102_DIGITS: [u8; 3] = [1, 0, 2]];
    static_unsigned_dcnumber![N103; N103_DIGITS: [u8; 3] = [1, 0, 3]];
    static_unsigned_dcnumber![N104; N104_DIGITS: [u8; 3] = [1, 0, 4]];
    static_unsigned_dcnumber![N105; N105_DIGITS: [u8; 3] = [1, 0, 5]];
    static_unsigned_dcnumber![N106; N106_DIGITS: [u8; 3] = [1, 0, 6]];
    static_unsigned_dcnumber![N107; N107_DIGITS: [u8; 3] = [1, 0, 7]];
    static_unsigned_dcnumber![N108; N108_DIGITS: [u8; 3] = [1, 0, 8]];
    static_unsigned_dcnumber![N109; N109_DIGITS: [u8; 3] = [1, 0, 9]];
    // 110 - 119
    static_unsigned_dcnumber![N110; N110_DIGITS: [u8; 3] = [1, 1, 0]];
    static_unsigned_dcnumber![N111; N111_DIGITS: [u8; 3] = [1, 1, 1]];
    static_unsigned_dcnumber![N112; N112_DIGITS: [u8; 3] = [1, 1, 2]];
    static_unsigned_dcnumber![N113; N113_DIGITS: [u8; 3] = [1, 1, 3]];
    static_unsigned_dcnumber![N114; N114_DIGITS: [u8; 3] = [1, 1, 4]];
    static_unsigned_dcnumber![N115; N115_DIGITS: [u8; 3] = [1, 1, 5]];
    static_unsigned_dcnumber![N116; N116_DIGITS: [u8; 3] = [1, 1, 6]];
    static_unsigned_dcnumber![N117; N117_DIGITS: [u8; 3] = [1, 1, 7]];
    static_unsigned_dcnumber![N118; N118_DIGITS: [u8; 3] = [1, 1, 8]];
    static_unsigned_dcnumber![N119; N119_DIGITS: [u8; 3] = [1, 1, 9]];
    // 120 - 129
    static_unsigned_dcnumber![N120; N120_DIGITS: [u8; 3] = [1, 2, 0]];
    static_unsigned_dcnumber![N121; N121_DIGITS: [u8; 3] = [1, 2, 1]];
    static_unsigned_dcnumber![N122; N122_DIGITS: [u8; 3] = [1, 2, 2]];
    static_unsigned_dcnumber![N123; N123_DIGITS: [u8; 3] = [1, 2, 3]];
    static_unsigned_dcnumber![N124; N124_DIGITS: [u8; 3] = [1, 2, 4]];
    static_unsigned_dcnumber![N125; N125_DIGITS: [u8; 3] = [1, 2, 5]];
    static_unsigned_dcnumber![N126; N126_DIGITS: [u8; 3] = [1, 2, 6]];
    static_unsigned_dcnumber![N127; N127_DIGITS: [u8; 3] = [1, 2, 7]];
    static_unsigned_dcnumber![N128; N128_DIGITS: [u8; 3] = [1, 2, 8]];
    static_unsigned_dcnumber![N129; N129_DIGITS: [u8; 3] = [1, 2, 9]];
    // 130 - 139
    static_unsigned_dcnumber![N130; N130_DIGITS: [u8; 3] = [1, 3, 0]];
    static_unsigned_dcnumber![N131; N131_DIGITS: [u8; 3] = [1, 3, 1]];
    static_unsigned_dcnumber![N132; N132_DIGITS: [u8; 3] = [1, 3, 2]];
    static_unsigned_dcnumber![N133; N133_DIGITS: [u8; 3] = [1, 3, 3]];
    static_unsigned_dcnumber![N134; N134_DIGITS: [u8; 3] = [1, 3, 4]];
    static_unsigned_dcnumber![N135; N135_DIGITS: [u8; 3] = [1, 3, 5]];
    static_unsigned_dcnumber![N136; N136_DIGITS: [u8; 3] = [1, 3, 6]];
    static_unsigned_dcnumber![N137; N137_DIGITS: [u8; 3] = [1, 3, 7]];
    static_unsigned_dcnumber![N138; N138_DIGITS: [u8; 3] = [1, 3, 8]];
    static_unsigned_dcnumber![N139; N139_DIGITS: [u8; 3] = [1, 3, 9]];
    // 140 - 149
    static_unsigned_dcnumber![N140; N140_DIGITS: [u8; 3] = [1, 4, 0]];
    static_unsigned_dcnumber![N141; N141_DIGITS: [u8; 3] = [1, 4, 1]];
    static_unsigned_dcnumber![N142; N142_DIGITS: [u8; 3] = [1, 4, 2]];
    static_unsigned_dcnumber![N143; N143_DIGITS: [u8; 3] = [1, 4, 3]];
    static_unsigned_dcnumber![N144; N144_DIGITS: [u8; 3] = [1, 4, 4]];
    static_unsigned_dcnumber![N145; N145_DIGITS: [u8; 3] = [1, 4, 5]];
    static_unsigned_dcnumber![N146; N146_DIGITS: [u8; 3] = [1, 4, 6]];
    static_unsigned_dcnumber![N147; N147_DIGITS: [u8; 3] = [1, 4, 7]];
    static_unsigned_dcnumber![N148; N148_DIGITS: [u8; 3] = [1, 4, 8]];
    static_unsigned_dcnumber![N149; N149_DIGITS: [u8; 3] = [1, 4, 9]];
    // 150 - 159
    static_unsigned_dcnumber![N150; N150_DIGITS: [u8; 3] = [1, 5, 0]];
    static_unsigned_dcnumber![N151; N151_DIGITS: [u8; 3] = [1, 5, 1]];
    static_unsigned_dcnumber![N152; N152_DIGITS: [u8; 3] = [1, 5, 2]];
    static_unsigned_dcnumber![N153; N153_DIGITS: [u8; 3] = [1, 5, 3]];
    static_unsigned_dcnumber![N154; N154_DIGITS: [u8; 3] = [1, 5, 4]];
    static_unsigned_dcnumber![N155; N155_DIGITS: [u8; 3] = [1, 5, 5]];
    static_unsigned_dcnumber![N156; N156_DIGITS: [u8; 3] = [1, 5, 6]];
    static_unsigned_dcnumber![N157; N157_DIGITS: [u8; 3] = [1, 5, 7]];
    static_unsigned_dcnumber![N158; N158_DIGITS: [u8; 3] = [1, 5, 8]];
    static_unsigned_dcnumber![N159; N159_DIGITS: [u8; 3] = [1, 5, 9]];
    // 160 - 169
    static_unsigned_dcnumber![N160; N160_DIGITS: [u8; 3] = [1, 6, 0]];
    static_unsigned_dcnumber![N161; N161_DIGITS: [u8; 3] = [1, 6, 1]];
    static_unsigned_dcnumber![N162; N162_DIGITS: [u8; 3] = [1, 6, 2]];
    static_unsigned_dcnumber![N163; N163_DIGITS: [u8; 3] = [1, 6, 3]];
    static_unsigned_dcnumber![N164; N164_DIGITS: [u8; 3] = [1, 6, 4]];
    static_unsigned_dcnumber![N165; N165_DIGITS: [u8; 3] = [1, 6, 5]];
    static_unsigned_dcnumber![N166; N166_DIGITS: [u8; 3] = [1, 6, 6]];
    static_unsigned_dcnumber![N167; N167_DIGITS: [u8; 3] = [1, 6, 7]];
    static_unsigned_dcnumber![N168; N168_DIGITS: [u8; 3] = [1, 6, 8]];
    static_unsigned_dcnumber![N169; N169_DIGITS: [u8; 3] = [1, 6, 9]];
    // 170 - 179
    static_unsigned_dcnumber![N170; N170_DIGITS: [u8; 3] = [1, 7, 0]];
    static_unsigned_dcnumber![N171; N171_DIGITS: [u8; 3] = [1, 7, 1]];
    static_unsigned_dcnumber![N172; N172_DIGITS: [u8; 3] = [1, 7, 2]];
    static_unsigned_dcnumber![N173; N173_DIGITS: [u8; 3] = [1, 7, 3]];
    static_unsigned_dcnumber![N174; N174_DIGITS: [u8; 3] = [1, 7, 4]];
    static_unsigned_dcnumber![N175; N175_DIGITS: [u8; 3] = [1, 7, 5]];
    static_unsigned_dcnumber![N176; N176_DIGITS: [u8; 3] = [1, 7, 6]];
    static_unsigned_dcnumber![N177; N177_DIGITS: [u8; 3] = [1, 7, 7]];
    static_unsigned_dcnumber![N178; N178_DIGITS: [u8; 3] = [1, 7, 8]];
    static_unsigned_dcnumber![N179; N179_DIGITS: [u8; 3] = [1, 7, 9]];
    // 180 - 189
    static_unsigned_dcnumber![N180; N180_DIGITS: [u8; 3] = [1, 8, 0]];
    static_unsigned_dcnumber![N181; N181_DIGITS: [u8; 3] = [1, 8, 1]];
    static_unsigned_dcnumber![N182; N182_DIGITS: [u8; 3] = [1, 8, 2]];
    static_unsigned_dcnumber![N183; N183_DIGITS: [u8; 3] = [1, 8, 3]];
    static_unsigned_dcnumber![N184; N184_DIGITS: [u8; 3] = [1, 8, 4]];
    static_unsigned_dcnumber![N185; N185_DIGITS: [u8; 3] = [1, 8, 5]];
    static_unsigned_dcnumber![N186; N186_DIGITS: [u8; 3] = [1, 8, 6]];
    static_unsigned_dcnumber![N187; N187_DIGITS: [u8; 3] = [1, 8, 7]];
    static_unsigned_dcnumber![N188; N188_DIGITS: [u8; 3] = [1, 8, 8]];
    static_unsigned_dcnumber![N189; N189_DIGITS: [u8; 3] = [1, 8, 9]];
    // 190 - 199
    static_unsigned_dcnumber![N190; N190_DIGITS: [u8; 3] = [1, 9, 0]];
    static_unsigned_dcnumber![N191; N191_DIGITS: [u8; 3] = [1, 9, 1]];
    static_unsigned_dcnumber![N192; N192_DIGITS: [u8; 3] = [1, 9, 2]];
    static_unsigned_dcnumber![N193; N193_DIGITS: [u8; 3] = [1, 9, 3]];
    static_unsigned_dcnumber![N194; N194_DIGITS: [u8; 3] = [1, 9, 4]];
    static_unsigned_dcnumber![N195; N195_DIGITS: [u8; 3] = [1, 9, 5]];
    static_unsigned_dcnumber![N196; N196_DIGITS: [u8; 3] = [1, 9, 6]];
    static_unsigned_dcnumber![N197; N197_DIGITS: [u8; 3] = [1, 9, 7]];
    static_unsigned_dcnumber![N198; N198_DIGITS: [u8; 3] = [1, 9, 8]];
    static_unsigned_dcnumber![N199; N199_DIGITS: [u8; 3] = [1, 9, 9]];
    // 200 - 209
    static_unsigned_dcnumber![N200; N200_DIGITS: [u8; 3] = [2, 0, 0]];
    static_unsigned_dcnumber![N201; N201_DIGITS: [u8; 3] = [2, 0, 1]];
    static_unsigned_dcnumber![N202; N202_DIGITS: [u8; 3] = [2, 0, 2]];
    static_unsigned_dcnumber![N203; N203_DIGITS: [u8; 3] = [2, 0, 3]];
    static_unsigned_dcnumber![N204; N204_DIGITS: [u8; 3] = [2, 0, 4]];
    static_unsigned_dcnumber![N205; N205_DIGITS: [u8; 3] = [2, 0, 5]];
    static_unsigned_dcnumber![N206; N206_DIGITS: [u8; 3] = [2, 0, 6]];
    static_unsigned_dcnumber![N207; N207_DIGITS: [u8; 3] = [2, 0, 7]];
    static_unsigned_dcnumber![N208; N208_DIGITS: [u8; 3] = [2, 0, 8]];
    static_unsigned_dcnumber![N209; N209_DIGITS: [u8; 3] = [2, 0, 9]];
    // 210 - 219
    static_unsigned_dcnumber![N210; N210_DIGITS: [u8; 3] = [2, 1, 0]];
    static_unsigned_dcnumber![N211; N211_DIGITS: [u8; 3] = [2, 1, 1]];
    static_unsigned_dcnumber![N212; N212_DIGITS: [u8; 3] = [2, 1, 2]];
    static_unsigned_dcnumber![N213; N213_DIGITS: [u8; 3] = [2, 1, 3]];
    static_unsigned_dcnumber![N214; N214_DIGITS: [u8; 3] = [2, 1, 4]];
    static_unsigned_dcnumber![N215; N215_DIGITS: [u8; 3] = [2, 1, 5]];
    static_unsigned_dcnumber![N216; N216_DIGITS: [u8; 3] = [2, 1, 6]];
    static_unsigned_dcnumber![N217; N217_DIGITS: [u8; 3] = [2, 1, 7]];
    static_unsigned_dcnumber![N218; N218_DIGITS: [u8; 3] = [2, 1, 8]];
    static_unsigned_dcnumber![N219; N219_DIGITS: [u8; 3] = [2, 1, 9]];
    // 220 - 229
    static_unsigned_dcnumber![N220; N220_DIGITS: [u8; 3] = [2, 2, 0]];
    static_unsigned_dcnumber![N221; N221_DIGITS: [u8; 3] = [2, 2, 1]];
    static_unsigned_dcnumber![N222; N222_DIGITS: [u8; 3] = [2, 2, 2]];
    static_unsigned_dcnumber![N223; N223_DIGITS: [u8; 3] = [2, 2, 3]];
    static_unsigned_dcnumber![N224; N224_DIGITS: [u8; 3] = [2, 2, 4]];
    static_unsigned_dcnumber![N225; N225_DIGITS: [u8; 3] = [2, 2, 5]];
    static_unsigned_dcnumber![N226; N226_DIGITS: [u8; 3] = [2, 2, 6]];
    static_unsigned_dcnumber![N227; N227_DIGITS: [u8; 3] = [2, 2, 7]];
    static_unsigned_dcnumber![N228; N228_DIGITS: [u8; 3] = [2, 2, 8]];
    static_unsigned_dcnumber![N229; N229_DIGITS: [u8; 3] = [2, 2, 9]];
    // 230 - 239
    static_unsigned_dcnumber![N230; N230_DIGITS: [u8; 3] = [2, 3, 0]];
    static_unsigned_dcnumber![N231; N231_DIGITS: [u8; 3] = [2, 3, 1]];
    static_unsigned_dcnumber![N232; N232_DIGITS: [u8; 3] = [2, 3, 2]];
    static_unsigned_dcnumber![N233; N233_DIGITS: [u8; 3] = [2, 3, 3]];
    static_unsigned_dcnumber![N234; N234_DIGITS: [u8; 3] = [2, 3, 4]];
    static_unsigned_dcnumber![N235; N235_DIGITS: [u8; 3] = [2, 3, 5]];
    static_unsigned_dcnumber![N236; N236_DIGITS: [u8; 3] = [2, 3, 6]];
    static_unsigned_dcnumber![N237; N237_DIGITS: [u8; 3] = [2, 3, 7]];
    static_unsigned_dcnumber![N238; N238_DIGITS: [u8; 3] = [2, 3, 8]];
    static_unsigned_dcnumber![N239; N239_DIGITS: [u8; 3] = [2, 3, 9]];
    // 240 - 249
    static_unsigned_dcnumber![N240; N240_DIGITS: [u8; 3] = [2, 4, 0]];
    static_unsigned_dcnumber![N241; N241_DIGITS: [u8; 3] = [2, 4, 1]];
    static_unsigned_dcnumber![N242; N242_DIGITS: [u8; 3] = [2, 4, 2]];
    static_unsigned_dcnumber![N243; N243_DIGITS: [u8; 3] = [2, 4, 3]];
    static_unsigned_dcnumber![N244; N244_DIGITS: [u8; 3] = [2, 4, 4]];
    static_unsigned_dcnumber![N245; N245_DIGITS: [u8; 3] = [2, 4, 5]];
    static_unsigned_dcnumber![N246; N246_DIGITS: [u8; 3] = [2, 4, 6]];
    static_unsigned_dcnumber![N247; N247_DIGITS: [u8; 3] = [2, 4, 7]];
    static_unsigned_dcnumber![N248; N248_DIGITS: [u8; 3] = [2, 4, 8]];
    static_unsigned_dcnumber![N249; N249_DIGITS: [u8; 3] = [2, 4, 9]];
    // 250 - 259
    static_unsigned_dcnumber![N250; N250_DIGITS: [u8; 3] = [2, 5, 0]];
    static_unsigned_dcnumber![N251; N251_DIGITS: [u8; 3] = [2, 5, 1]];
    static_unsigned_dcnumber![N252; N252_DIGITS: [u8; 3] = [2, 5, 2]];
    static_unsigned_dcnumber![N253; N253_DIGITS: [u8; 3] = [2, 5, 3]];
    static_unsigned_dcnumber![N254; N254_DIGITS: [u8; 3] = [2, 5, 4]];
    static_unsigned_dcnumber![N255; N255_DIGITS: [u8; 3] = [2, 5, 5]];

    static SMALL_INTS: [&UnsignedDCNumber;256] = [
        &N0, &N1, &N2, &N3, &N4, &N5, &N6, &N7, &N8, &N9,
        &N10, &N11, &N12, &N13, &N14, &N15, &N16, &N17, &N18, &N19,
        &N20, &N21, &N22, &N23, &N24, &N25, &N26, &N27, &N28, &N29,
        &N30, &N31, &N32, &N33, &N34, &N35, &N36, &N37, &N38, &N39,
        &N40, &N41, &N42, &N43, &N44, &N45, &N46, &N47, &N48, &N49,
        &N50, &N51, &N52, &N53, &N54, &N55, &N56, &N57, &N58, &N59,
        &N60, &N61, &N62, &N63, &N64, &N65, &N66, &N67, &N68, &N69,
        &N70, &N71, &N72, &N73, &N74, &N75, &N76, &N77, &N78, &N79,
        &N80, &N81, &N82, &N83, &N84, &N85, &N86, &N87, &N88, &N89,
        &N90, &N91, &N92, &N93, &N94, &N95, &N96, &N97, &N98, &N99,
        &N100, &N101, &N102, &N103, &N104, &N105, &N106, &N107, &N108, &N109,
        &N110, &N111, &N112, &N113, &N114, &N115, &N116, &N117, &N118, &N119,
        &N120, &N121, &N122, &N123, &N124, &N125, &N126, &N127, &N128, &N129,
        &N130, &N131, &N132, &N133, &N134, &N135, &N136, &N137, &N138, &N139,
        &N140, &N141, &N142, &N143, &N144, &N145, &N146, &N147, &N148, &N149,
        &N150, &N151, &N152, &N153, &N154, &N155, &N156, &N157, &N158, &N159,
        &N160, &N161, &N162, &N163, &N164, &N165, &N166, &N167, &N168, &N169,
        &N170, &N171, &N172, &N173, &N174, &N175, &N176, &N177, &N178, &N179,
        &N180, &N181, &N182, &N183, &N184, &N185, &N186, &N187, &N188, &N189,
        &N190, &N191, &N192, &N193, &N194, &N195, &N196, &N197, &N198, &N199,
        &N200, &N201, &N202, &N203, &N204, &N205, &N206, &N207, &N208, &N209,
        &N210, &N211, &N212, &N213, &N214, &N215, &N216, &N217, &N218, &N219,
        &N220, &N221, &N222, &N223, &N224, &N225, &N226, &N227, &N228, &N229,
        &N230, &N231, &N232, &N233, &N234, &N235, &N236, &N237, &N238, &N239,
        &N240, &N241, &N242, &N243, &N244, &N245, &N246, &N247, &N248, &N249,
        &N250, &N251, &N252, &N253, &N254, &N255
    ];

    pub fn interned<'a>(n: u8) -> UnsignedDCNumber<'static> {
        unsafe {
//            debug_assert!(u8::max_value() as usize <= SMALL_INTS.len());
            let r = *SMALL_INTS.get_unchecked(n as usize);
            r.clone()

        }
    }
}

impl<'a> UnsignedDCNumber<'a> {
    pub fn new<T>(digits: T, last_integer: usize) -> Self
        where
            Cow<'a, [u8]>: From<T>,
    {
        let v: Cow<[u8]> = digits.into();
        debug_assert!(
            last_integer <= v.len(),
            "separator {} should be less than {}: v{:?}",
            last_integer,
            v.len(),
            v
        );

        UnsignedDCNumber {
            digits: v,
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

    /// Creates UnsignecDCNumber from a byte representing a decimal ascii value
    ///
    /// ```
    /// use rdc::dcnumber::unsigned::UnsignedDCNumber;
    /// use rdc::dcnumber::error::ParseDCNumberError;
    /// use rdc::dcnumber::traits::FromBytes;
    ///
    /// assert_eq!(UnsignedDCNumber::from_bytes("0".as_ref()), UnsignedDCNumber::from_byte(b'0'));
    /// assert_eq!(UnsignedDCNumber::from_bytes("1".as_ref()), UnsignedDCNumber::from_byte(b'1'));
    /// ```
    ///
    pub fn from_byte(byte: u8) -> Result<Self, ParseDCNumberError> {
        UnsignedDCNumber::from_byte_radix(byte, 10)
    }

    /// Creates UnsignecDCNumber from a byte representing a decimal ascii value
    ///
    /// ```
    /// use rdc::dcnumber::unsigned::UnsignedDCNumber;
    /// use rdc::dcnumber::error::ParseDCNumberError;
    /// use rdc::dcnumber::traits::FromBytes;
    /// ```
    /// TODO put me back
    /// assert_eq!(UnsignedDCNumber::from_bytes_radix("0".as_ref(), 8), UnsignedDCNumber::from_byte_radix(b'0', 8));
    /// assert_eq!(UnsignedDCNumber::from_bytes_radix("0".as_ref(), 8), UnsignedDCNumber::from_byte_radix(b'0', 10));
    pub fn from_byte_radix(byte: u8, radix: u32) -> Result<Self, ParseDCNumberError> {
        if radix > 16 {
            return Err(ParseDCNumberError::InvalidRadix);
        }
        UnsignedDCNumber::from_byte_radix_u8(byte, radix as u8)
    }

    pub fn from_byte_radix_u8(byte: u8, radix: u8) -> Result<Self, ParseDCNumberError> {
        if radix < 2 || radix > 16 {
            return Err(ParseDCNumberError::InvalidRadix);
        }
        match byte {
            ch @ b'0'...b'9' if ch - b'0' < radix => Ok(small_ints::interned(ch - b'0')),
            ch @ b'A'...b'F' if ch - b'A' + 10 < radix => Ok(small_ints::interned(ch - b'A' + 10)),
            _ => Err(ParseDCNumberError::InvalidDigit),
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
        eprintln!("CMP DEBUG XXXXXXXX   self= {:?} other= {:?}", self, other);
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

struct DCNumberAlignment<'a> {
    leading_digits: &'a [u8],
    aligned_part: &'a [u8],
    second_aligned_part: &'a [u8],
    fractional_tail: &'a [u8],
}

impl<'a> DCNumberAlignment<'a> {
    fn align_ref(lhs: &'a UnsignedDCNumber, rhs: &'a UnsignedDCNumber) -> DCNumberAlignment<'a>
    {
        let leading_digits;
        let right_aligned_part;
        let fractional_tail;
        let aligned_part;
        let second_right_aligned_part: &[u8];
        let second_aligned_part: &[u8];

        if lhs.fractional_digits() > rhs.fractional_digits() {
            let offset = lhs.fractional_digits() - rhs.fractional_digits();
            let (front, tail) = lhs.digits.split_at(lhs.digits.len() - offset);
            right_aligned_part = front;
            fractional_tail = tail;
            second_right_aligned_part = &rhs.digits;
        } else {
            let offset = rhs.fractional_digits() - lhs.fractional_digits();
            let (front, tail) = rhs.digits.split_at(rhs.digits.len() - offset);
            right_aligned_part = front;
            fractional_tail = tail;
            second_right_aligned_part = &lhs.digits;
        }


        if right_aligned_part.len() > second_right_aligned_part.len() {
            let offset = right_aligned_part.len() - second_right_aligned_part.len();
            let (front, tail) = right_aligned_part.split_at(offset);
            aligned_part = tail;
            leading_digits = front;
            second_aligned_part = second_right_aligned_part;
        } else {
            let offset = second_right_aligned_part.len() - right_aligned_part.len();
            let (front, tail) = second_right_aligned_part.split_at(offset);
            aligned_part = tail;
            leading_digits = front;
            second_aligned_part = right_aligned_part;
        }

        DCNumberAlignment { leading_digits, aligned_part, second_aligned_part, fractional_tail }
    }

    fn len(&self) -> usize {
        self.fractional_tail.len() + self.aligned_part.len() + self.leading_digits.len()
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
        // TODO optimization for 0 and powers of 10...

        let mut separator = max(self.separator, other.separator);
        let alignment = DCNumberAlignment::align_ref(&self, &other);
        let total_len = alignment.len();
        let DCNumberAlignment { leading_digits, aligned_part, second_aligned_part, fractional_tail } = alignment;

        let mut carry = false;
        let mut digits: Vec<u8> = Vec::new();


        digits.extend(fractional_tail.iter().rev());
        for (lhs, rhs) in aligned_part.iter().rev().cloned().zip(second_aligned_part.iter().rev().cloned()) {
            debug_assert!(lhs < 10);
            debug_assert!(rhs < 10);
            let sum = if carry {
                carry = false;
                lhs + rhs + 1 // no risk of overflow, both < 10
            } else {
                lhs + rhs
            };
            let mut result = sum;
            if sum >= 10 {
                carry = true;
                result -= 10;
            }
            digits.push(result);
        }

        for &digit in leading_digits.iter().rev() {
            debug_assert!(digit < 10);
            let value = if carry {
                digit + 1
            } else {
                digit
            };

            let mut result = value;
            if result >= 10 {
                carry = true;
                result -= 10;
            }
            digits.push(result);
        }

        if carry {
            separator += 1;
            digits.push(1);
        }

        digits.reverse();

        UnsignedDCNumber::new(digits, separator)
    }
}

// TODO consider if implementing Add<&> allows us faster

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

macro_rules! impl_from_unsigned_primitive {
    ($u:ty) => {
        impl<'a> From<$u> for UnsignedDCNumber<'a> {
            fn from(n: $u) -> Self {
                let n_digits = decimal_digits(n as u64) as usize;
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
    };
}

// From integer types
/// Creates UnsignedDCNumber from unsigned integer
///
/// ```
/// use std::str::FromStr;
/// use rdc::dcnumber::unsigned::UnsignedDCNumber;
/// use rdc::dcnumber::error::ParseDCNumberError;
///
/// assert_eq!(UnsignedDCNumber::from_str("0").unwrap(), UnsignedDCNumber::from(0 as u8));
/// assert_eq!(UnsignedDCNumber::from_str("1").unwrap(), UnsignedDCNumber::from(1 as u8));
/// assert_eq!(UnsignedDCNumber::from_str("10").unwrap(), UnsignedDCNumber::from(10 as u8));
/// assert_eq!(UnsignedDCNumber::from_str("110").unwrap(), UnsignedDCNumber::from(110 as u8));
/// assert_eq!(UnsignedDCNumber::from_str("255").unwrap(), UnsignedDCNumber::from(255 as u8));
/// ```
///
impl<'a> From<u8> for UnsignedDCNumber<'a> {
    fn from(n: u8) -> Self {
        small_ints::interned(n)
    }
}

/// Creates UnsignedDCNumber from unsigned integer
///
/// ```
/// use rdc::dcnumber::unsigned::UnsignedDCNumber;
/// use rdc::dcnumber::error::ParseDCNumberError;
///
/// assert_eq!(UnsignedDCNumber::from_str("0").unwrap(), UnsignedDCNumber::from(0 as u16));
/// assert_eq!(UnsignedDCNumber::from_str("1").unwrap(), UnsignedDCNumber::from(1 as u16));
/// assert_eq!(UnsignedDCNumber::from_str("10").unwrap(), UnsignedDCNumber::from(10 as u16));
/// assert_eq!(UnsignedDCNumber::from_str("110").unwrap(), UnsignedDCNumber::from(110 as u16));
/// assert_eq!(UnsignedDCNumber::from_str("255").unwrap(), UnsignedDCNumber::from(255 as u16));
/// ```
impl_from_unsigned_primitive![u16];

/// Creates UnsignedDCNumber from unsigned integer
///
/// ```
/// use rdc::dcnumber::unsigned::UnsignedDCNumber;
/// use rdc::dcnumber::error::ParseDCNumberError;
///
/// assert_eq!(UnsignedDCNumber::from_str("0").unwrap(), UnsignedDCNumber::from(0 as u32));
/// assert_eq!(UnsignedDCNumber::from_str("1").unwrap(), UnsignedDCNumber::from(1 as u32));
/// assert_eq!(UnsignedDCNumber::from_str("10").unwrap(), UnsignedDCNumber::from(10 as u32));
/// assert_eq!(UnsignedDCNumber::from_str("110").unwrap(), UnsignedDCNumber::from(110 as u32));
/// assert_eq!(UnsignedDCNumber::from_str("255").unwrap(), UnsignedDCNumber::from(255 as u32));
/// ```
impl_from_unsigned_primitive![u32];

/// Creates UnsignedDCNumber from unsigned integer
///
/// ```
/// use rdc::dcnumber::unsigned::UnsignedDCNumber;
/// use rdc::dcnumber::error::ParseDCNumberError;
///
/// assert_eq!(UnsignedDCNumber::from_str("0").unwrap(), UnsignedDCNumber::from(0 as u64));
/// assert_eq!(UnsignedDCNumber::from_str("1").unwrap(), UnsignedDCNumber::from(1 as u64));
/// assert_eq!(UnsignedDCNumber::from_str("10").unwrap(), UnsignedDCNumber::from(10 as u64));
/// assert_eq!(UnsignedDCNumber::from_str("110").unwrap(), UnsignedDCNumber::from(110 as u64));
/// assert_eq!(UnsignedDCNumber::from_str("255").unwrap(), UnsignedDCNumber::from(255 as u64));
/// ```
impl_from_unsigned_primitive![u64];

mod radix_converters {
    use super::{ParseDCNumberError, UnsignedDCNumber, ZERO};

    pub trait AsciiConverter {
        fn convert_bytes<'a, 'b>(
            &self,
            bytes: &'a [u8],
        ) -> Result<UnsignedDCNumber<'b>, ParseDCNumberError>;
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

    impl AsciiConverter for DecAsciiConverter {
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
    pub struct RadixAsciiConverter {
        radix: u8,
    }

    impl RadixAsciiConverter {
        pub fn new(radix: u8) -> Self {
            assert!(radix <= 16 && radix >= 2);
            Self { radix }
        }
    }

    impl AsciiConverter for RadixAsciiConverter {
        fn convert_bytes<'a, 'b>(
            &self,
            bytes: &'a [u8],
        ) -> Result<UnsignedDCNumber<'b>, ParseDCNumberError> {
            let radix = self.radix;
            bytes.iter().fold(Ok(ZERO.clone()), |acc, &ch| {
                acc.and_then(|n| {
                    UnsignedDCNumber::from_byte_radix_u8(ch, radix).and_then(|m| Ok(m + (n * 10)))
                })
            })
        }
    }
}

impl<'a> FromBytes for UnsignedDCNumber<'a> {
    type Err = ParseDCNumberError;

    fn from_bytes_radix(bytes: &[u8], radix: u32) -> Result<Self, ParseDCNumberError> {
        use self::radix_converters::AsciiConverter;

        match radix {
            2...9 => radix_converters::RadixAsciiConverter::new(radix as u8).convert_bytes(bytes),
            10 => radix_converters::DecAsciiConverter::new().convert_bytes(bytes),
            11...16 => radix_converters::RadixAsciiConverter::new(radix as u8).convert_bytes(bytes),
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

    #[test]
    fn test_align1() {
        let n = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let m = UnsignedDCNumber::new([7, 8, 9, 2].as_ref(), 3); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        assert_eq!([1, ].as_ref(), alignment.leading_digits);
        assert_eq!([2, 3, 4, 5].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2].as_ref(), alignment.second_aligned_part);
        assert_eq!([6].as_ref(), alignment.fractional_tail);
    }


    #[test]
    fn test_align2() {
        let n = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let m = UnsignedDCNumber::new([7, 8, 9, 2, 3, 4, 5].as_ref(), 3); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        assert_eq!([1, ].as_ref(), alignment.leading_digits);
        assert_eq!([2, 3, 4, 5, 6].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2, 3].as_ref(), alignment.second_aligned_part);
        assert_eq!([4, 5].as_ref(), alignment.fractional_tail);
    }


    #[test]
    fn test_align3() {
        let n = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let m = UnsignedDCNumber::new([7, 8, 9, 2, 3, 4, 5].as_ref(), 4); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        let empty: &[u8] = &[];
        assert_eq!(empty, alignment.leading_digits);
        assert_eq!([1, 2, 3, 4, 5, 6].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2, 3, 4].as_ref(), alignment.second_aligned_part);
        assert_eq!([5].as_ref(), alignment.fractional_tail);
    }

    #[test]
    fn test_align4() {
        let n = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let m = UnsignedDCNumber::new([7, 8, 9, 2, 3, 4].as_ref(), 4); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        let empty: &[u8] = &[];
        assert_eq!(empty, alignment.leading_digits);
        assert_eq!([1, 2, 3, 4, 5, 6].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2, 3, 4].as_ref(), alignment.second_aligned_part);
        assert_eq!(empty, alignment.fractional_tail);
    }

    #[test]
    fn test_align5() {
        let m = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let n = UnsignedDCNumber::new([7, 8, 9, 2].as_ref(), 3); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        assert_eq!([1, ].as_ref(), alignment.leading_digits);
        assert_eq!([2, 3, 4, 5].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2].as_ref(), alignment.second_aligned_part);
        assert_eq!([6].as_ref(), alignment.fractional_tail);
    }


    #[test]
    fn test_align6() {
        let m = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let n = UnsignedDCNumber::new([7, 8, 9, 2, 3, 4, 5].as_ref(), 3); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        assert_eq!([1, ].as_ref(), alignment.leading_digits);
        assert_eq!([2, 3, 4, 5, 6].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2, 3].as_ref(), alignment.second_aligned_part);
        assert_eq!([4, 5].as_ref(), alignment.fractional_tail);
    }


    #[test]
    fn test_align7() {
        let m = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let n = UnsignedDCNumber::new([7, 8, 9, 2, 3, 4, 5].as_ref(), 4); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        let empty: &[u8] = &[];
        assert_eq!(empty, alignment.leading_digits);
        assert_eq!([1, 2, 3, 4, 5, 6].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2, 3, 4].as_ref(), alignment.second_aligned_part);
        assert_eq!([5].as_ref(), alignment.fractional_tail);
    }

    #[test]
    fn test_align8() {
        let n = UnsignedDCNumber::new([1, 2, 3, 4, 5, 6].as_ref(), 4); // 1234.56
        let m = UnsignedDCNumber::new([7, 8, 9, 2, 3, 4].as_ref(), 4); // 789.2

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        let empty: &[u8] = &[];
        assert_eq!(empty, alignment.leading_digits);
        assert_eq!([1, 2, 3, 4, 5, 6].as_ref(), alignment.aligned_part);
        assert_eq!([7, 8, 9, 2, 3, 4].as_ref(), alignment.second_aligned_part);
        assert_eq!(empty, alignment.fractional_tail);
    }

    #[test]
    fn test_align9() {
        let n = UnsignedDCNumber::new([5, 2, 0].as_ref(), 3);
        let m = UnsignedDCNumber::new([5, 2, 6].as_ref(), 3);

        let alignment = DCNumberAlignment::align_ref(&n, &m);

        let empty: &[u8] = &[];
        assert_eq!(empty, alignment.leading_digits);
        assert_eq!([5, 2, 0].as_ref(), alignment.aligned_part);
        assert_eq!([5, 2, 6].as_ref(), alignment.second_aligned_part);
        assert_eq!(empty, alignment.fractional_tail);
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

    macro_rules! test_from_byte_radix {
        ($test_name:ident : $digits:tt; $radix:expr) => {
            #[test]
            fn $test_name() {
                assert_eq!(
                    UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), $radix),
                    UnsignedDCNumber::from_byte_radix(stringify!($digits).as_bytes()[0], $radix)
                )
            }
        };

        (
            $test_name:ident : $lhs_digits:expr, $lhs_radix:expr; $rhs_digits:expr, $rhs_radix:expr
        ) => {
            #[test]
            fn $test_name() {
                assert_eq!(
                    UnsignedDCNumber::from_byte_radix($lhs_digits, $lhs_radix),
                    UnsignedDCNumber::from_byte_radix($rhs_digits, $rhs_radix),
                );
            }
        };
    }

    test_from_byte_radix!(from_byte_radix_1_8: 1; 8);
    test_from_byte_radix!(from_byte_radix_2_8: 2; 8);
    test_from_byte_radix!(from_byte_radix_3_8: 3; 8);
    test_from_byte_radix!(from_byte_radix_4_8: 4; 8);
    test_from_byte_radix!(from_byte_radix_5_8: 5; 8);
    test_from_byte_radix!(from_byte_radix_6_8: 6; 8);
    test_from_byte_radix!(from_byte_radix_7_8: 7; 8);
    test_from_byte_radix!(from_byte_radix_8_8: 8; 8);
    test_from_byte_radix!(from_byte_radix_9_8: 9; 8);
    test_from_byte_radix!(from_byte_radix_8_10: 8; 10);
    test_from_byte_radix!(from_byte_radix_9_10: 9; 10);
    test_from_byte_radix!(from_byte_radix_A_8: A; 8);
    test_from_byte_radix!(from_byte_radix_A_10: A; 10);
    test_from_byte_radix!(from_byte_radix_A_16: A; 16);

    fn test_regression_A_16() {
        let n = UnsignedDCNumber::from_str_radix("A", 16).expect("A in hex should be fine");
        assert_eq!(UnsignedDCNumber::new([1, 0].as_ref(), 2), n);
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
            UnsignedDCNumber::from(213 as u32)
                .partial_cmp(&UnsignedDCNumber::from_str("321.12").unwrap())
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
    test_binop![test_add_le:10.1 = 9.9 + 0.2];
    test_binop![test_add_le2:10.12 = 9.9 + 0.22];
    test_binop![test_add_le3:10.12 = 9.92 + 0.2];
    test_binop![test_add_le4:10.12 = 0.92 + 9.2];
    test_binop![test_add_le5:1000.12 = 990.92 + 9.2];
    test_binop![test_add_le6:1000.12 = 999.92 + 0.2];

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
        let zero = UnsignedDCNumber::from(0 as u64);
        assert_eq!(ZERO, zero);
    }

    #[test]
    fn test_from_u64_10() {
        let n = UnsignedDCNumber::from(10 as u64);
        assert_eq!(UnsignedDCNumber::new([1, 0].as_ref(), 2), n);
    }

    #[test]
    fn test_from_u64_one() {
        let one = UnsignedDCNumber::from(1 as u64);
        assert_eq!(ONE, one);
    }

    #[test]
    fn test_from_u64() {
        let n = UnsignedDCNumber::from(1234567890 as u64);
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

    test_from_str![from_str_zero: ZERO ; 0];
    test_from_str![from_str_one:  ONE ; 1];
    test_from_str![from_str_10: UnsignedDCNumber::new([1, 0].as_ref(), 2) ; 10];
    test_from_str![from_str_byte_spec: UnsignedDCNumber::new([1, 1].as_ref(), 1) ; 1.1];
    test_from_str![from_str_0dot9: UnsignedDCNumber::new([0, 9].as_ref(), 1) ; 0.9];
    test_from_str![from_str_1000dot3: UnsignedDCNumber::new([1, 0, 0, 0, 3].as_ref(), 4) ; 1000.3];
    test_from_str![from_str_0dot01: UnsignedDCNumber::new([0, 0, 1].as_ref(), 1) ; 0.01];
    test_from_str![from_str_from_int: UnsignedDCNumber::from(1234 as u16) ; 1234 ];
    test_from_str![from_str_from_int_leading0: UnsignedDCNumber::from(1234 as u16) ; 01234];
    test_from_str![from_str_empty : EmptyString <- ""];
    test_from_str![from_str_a : InvalidDigit <- "a"];
    test_from_str![from_str_1a : InvalidDigit <- "1a]"];
    test_from_str![from_str_0a : InvalidDigit <- "0a"];
    test_from_str![from_str_dota : InvalidDigit <- ".a"];
    test_from_str![from_str_0dotdot0: RepeatedDot <- "0..0"];
    test_eq![from_tail0 : 1234.32 = 1234.320 ];
    test_eq![from_taildot0 : 1234 = 1234.0 ];
    test_eq![from_ident : 1234 = 1234.];
    test_eq![from_leading0_f : 01234.32 = 1234.32 ];
    test_eq![from_leading_tailing_0f : 01234.32 = 1234.320 ];
    test_eq![eq_zero: 0 = 0];
    test_eq![eq_one: 1 = 1];
    test_eq![eq_one_dot_one: 1.1 = 1.1];
    test_eq![eq_0dot9: 0.9 = 0.9];
    test_eq![eq_0dot01: 0.01 = 0.01];
    test_eq![eq_1740: 1740 = 1740];
    test_eq![eq_1000dot3: 1000.3 = 1000.3];
    test_eq![eq_10: 10 = 10];

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

    fn test_from_u8() {
        use std::io::Write;
        for i in 0..256 {
            let i = i as u8;
            let mut out = Vec::new();

            assert_eq!(i as u64, UnsignedDCNumber::from(i).to_u64().unwrap());

            let _ = write!(out, "{}", UnsignedDCNumber::from(i)).expect("write");
            assert_eq!(i.to_string(), String::from_utf8(out).expect("utf8 issue"), )
        }
    }

    // TODO write test for from_bytes with various bases

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
        from_bytes_radix![first_hex: 10 = A: 16];
//        from_bytes_radix![b2_10: 2 = 10: 2];
//        from_bytes_radix![b3_10: 3 = 10: 3];
//        from_bytes_radix![b4_10: 4 = 10: 4];
//        from_bytes_radix![b5_10: 5 = 10: 5];
//        from_bytes_radix![b6_10: 6 = 10: 6];
//        from_bytes_radix![b7_10: 7 = 10: 7];
//        from_bytes_radix![b8_10: 8 = 10: 8];
//        from_bytes_radix![b9_10: 9 = 10: 9];
    from_bytes_radix![b8_0: 0 = 0: 8];
    from_bytes_radix![b8_1: 1 = 1: 8];
    from_bytes_radix![b8_2: 2 = 2: 8];
    from_bytes_radix![b10_0: 0 = 0: 10];
    from_bytes_radix![b10_1: 1 = 1: 10];
    from_bytes_radix![b10_2: 2 = 2: 10];
    from_bytes_radix![b16_0: 0 = 0: 16];
    from_bytes_radix![b16_1: 1 = 1: 16];
    from_bytes_radix![b16_2: 2 = 2: 16];
    from_bytes_radix![b16_A: 10 = A: 16];

    bench_from_str![short_int: "3"];
    bench_from_str![mid_int: "17235428"];
    bench_from_str![long_int: "172354283422734622371431236441234351267438543781453193415694871634731457681354784531"];
    bench_from_str![longer_int: "17235428342273462237143123644123435126743854378145319341569487000000000000163473145768135478453123187356412946123041213310238698752341280000000000000000000000"];
}
