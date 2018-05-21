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

    pub fn interned<'a>(n: u8) -> UnsignedDCNumber<'a> {
        match n {
            0u8 => N0.clone(),
            1u8 => N1.clone(),
            2u8 => N2.clone(),
            3u8 => N3.clone(),
            4u8 => N4.clone(),
            5u8 => N5.clone(),
            6u8 => N6.clone(),
            7u8 => N7.clone(),
            8u8 => N8.clone(),
            9u8 => N9.clone(),
            10u8 => N10.clone(),
            11u8 => N11.clone(),
            12u8 => N12.clone(),
            13u8 => N13.clone(),
            14u8 => N14.clone(),
            15u8 => N15.clone(),
            16u8 => N16.clone(),
            17u8 => N17.clone(),
            18u8 => N18.clone(),
            19u8 => N19.clone(),
            20u8 => N20.clone(),
            21u8 => N21.clone(),
            22u8 => N22.clone(),
            23u8 => N23.clone(),
            24u8 => N24.clone(),
            25u8 => N25.clone(),
            26u8 => N26.clone(),
            27u8 => N27.clone(),
            28u8 => N28.clone(),
            29u8 => N29.clone(),
            30u8 => N30.clone(),
            31u8 => N31.clone(),
            32u8 => N32.clone(),
            33u8 => N33.clone(),
            34u8 => N34.clone(),
            35u8 => N35.clone(),
            36u8 => N36.clone(),
            37u8 => N37.clone(),
            38u8 => N38.clone(),
            39u8 => N39.clone(),
            40u8 => N40.clone(),
            41u8 => N41.clone(),
            42u8 => N42.clone(),
            43u8 => N43.clone(),
            44u8 => N44.clone(),
            45u8 => N45.clone(),
            46u8 => N46.clone(),
            47u8 => N47.clone(),
            48u8 => N48.clone(),
            49u8 => N49.clone(),
            50u8 => N50.clone(),
            51u8 => N51.clone(),
            52u8 => N52.clone(),
            53u8 => N53.clone(),
            54u8 => N54.clone(),
            55u8 => N55.clone(),
            56u8 => N56.clone(),
            57u8 => N57.clone(),
            58u8 => N58.clone(),
            59u8 => N59.clone(),
            60u8 => N60.clone(),
            61u8 => N61.clone(),
            62u8 => N62.clone(),
            63u8 => N63.clone(),
            64u8 => N64.clone(),
            65u8 => N65.clone(),
            66u8 => N66.clone(),
            67u8 => N67.clone(),
            68u8 => N68.clone(),
            69u8 => N69.clone(),
            70u8 => N70.clone(),
            71u8 => N71.clone(),
            72u8 => N72.clone(),
            73u8 => N73.clone(),
            74u8 => N74.clone(),
            75u8 => N75.clone(),
            76u8 => N76.clone(),
            77u8 => N77.clone(),
            78u8 => N78.clone(),
            79u8 => N79.clone(),
            80u8 => N80.clone(),
            81u8 => N81.clone(),
            82u8 => N82.clone(),
            83u8 => N83.clone(),
            84u8 => N84.clone(),
            85u8 => N85.clone(),
            86u8 => N86.clone(),
            87u8 => N87.clone(),
            88u8 => N88.clone(),
            89u8 => N89.clone(),
            90u8 => N90.clone(),
            91u8 => N91.clone(),
            92u8 => N92.clone(),
            93u8 => N93.clone(),
            94u8 => N94.clone(),
            95u8 => N95.clone(),
            96u8 => N96.clone(),
            97u8 => N97.clone(),
            98u8 => N98.clone(),
            99u8 => N99.clone(),
            100u8 => N100.clone(),
            101u8 => N101.clone(),
            102u8 => N102.clone(),
            103u8 => N103.clone(),
            104u8 => N104.clone(),
            105u8 => N105.clone(),
            106u8 => N106.clone(),
            107u8 => N107.clone(),
            108u8 => N108.clone(),
            109u8 => N109.clone(),
            110u8 => N110.clone(),
            111u8 => N111.clone(),
            112u8 => N112.clone(),
            113u8 => N113.clone(),
            114u8 => N114.clone(),
            115u8 => N115.clone(),
            116u8 => N116.clone(),
            117u8 => N117.clone(),
            118u8 => N118.clone(),
            119u8 => N119.clone(),
            120u8 => N120.clone(),
            121u8 => N121.clone(),
            122u8 => N122.clone(),
            123u8 => N123.clone(),
            124u8 => N124.clone(),
            125u8 => N125.clone(),
            126u8 => N126.clone(),
            127u8 => N127.clone(),
            128u8 => N128.clone(),
            129u8 => N129.clone(),
            130u8 => N130.clone(),
            131u8 => N131.clone(),
            132u8 => N132.clone(),
            133u8 => N133.clone(),
            134u8 => N134.clone(),
            135u8 => N135.clone(),
            136u8 => N136.clone(),
            137u8 => N137.clone(),
            138u8 => N138.clone(),
            139u8 => N139.clone(),
            140u8 => N140.clone(),
            141u8 => N141.clone(),
            142u8 => N142.clone(),
            143u8 => N143.clone(),
            144u8 => N144.clone(),
            145u8 => N145.clone(),
            146u8 => N146.clone(),
            147u8 => N147.clone(),
            148u8 => N148.clone(),
            149u8 => N149.clone(),
            150u8 => N150.clone(),
            151u8 => N151.clone(),
            152u8 => N152.clone(),
            153u8 => N153.clone(),
            154u8 => N154.clone(),
            155u8 => N155.clone(),
            156u8 => N156.clone(),
            157u8 => N157.clone(),
            158u8 => N158.clone(),
            159u8 => N159.clone(),
            160u8 => N160.clone(),
            161u8 => N161.clone(),
            162u8 => N162.clone(),
            163u8 => N163.clone(),
            164u8 => N164.clone(),
            165u8 => N165.clone(),
            166u8 => N166.clone(),
            167u8 => N167.clone(),
            168u8 => N168.clone(),
            169u8 => N169.clone(),
            170u8 => N170.clone(),
            171u8 => N171.clone(),
            172u8 => N172.clone(),
            173u8 => N173.clone(),
            174u8 => N174.clone(),
            175u8 => N175.clone(),
            176u8 => N176.clone(),
            177u8 => N177.clone(),
            178u8 => N178.clone(),
            179u8 => N179.clone(),
            180u8 => N180.clone(),
            181u8 => N181.clone(),
            182u8 => N182.clone(),
            183u8 => N183.clone(),
            184u8 => N184.clone(),
            185u8 => N185.clone(),
            186u8 => N186.clone(),
            187u8 => N187.clone(),
            188u8 => N188.clone(),
            189u8 => N189.clone(),
            190u8 => N190.clone(),
            191u8 => N191.clone(),
            192u8 => N192.clone(),
            193u8 => N193.clone(),
            194u8 => N194.clone(),
            195u8 => N195.clone(),
            196u8 => N196.clone(),
            197u8 => N197.clone(),
            198u8 => N198.clone(),
            199u8 => N199.clone(),
            200u8 => N200.clone(),
            201u8 => N201.clone(),
            202u8 => N202.clone(),
            203u8 => N203.clone(),
            204u8 => N204.clone(),
            205u8 => N205.clone(),
            206u8 => N206.clone(),
            207u8 => N207.clone(),
            208u8 => N208.clone(),
            209u8 => N209.clone(),
            210u8 => N210.clone(),
            211u8 => N211.clone(),
            212u8 => N212.clone(),
            213u8 => N213.clone(),
            214u8 => N214.clone(),
            215u8 => N215.clone(),
            216u8 => N216.clone(),
            217u8 => N217.clone(),
            218u8 => N218.clone(),
            219u8 => N219.clone(),
            220u8 => N220.clone(),
            221u8 => N221.clone(),
            222u8 => N222.clone(),
            223u8 => N223.clone(),
            224u8 => N224.clone(),
            225u8 => N225.clone(),
            226u8 => N226.clone(),
            227u8 => N227.clone(),
            228u8 => N228.clone(),
            229u8 => N229.clone(),
            230u8 => N230.clone(),
            231u8 => N231.clone(),
            232u8 => N232.clone(),
            233u8 => N233.clone(),
            234u8 => N234.clone(),
            235u8 => N235.clone(),
            236u8 => N236.clone(),
            237u8 => N237.clone(),
            238u8 => N238.clone(),
            239u8 => N239.clone(),
            240u8 => N240.clone(),
            241u8 => N241.clone(),
            242u8 => N242.clone(),
            243u8 => N243.clone(),
            244u8 => N244.clone(),
            245u8 => N245.clone(),
            246u8 => N246.clone(),
            247u8 => N247.clone(),
            248u8 => N248.clone(),
            249u8 => N249.clone(),
            250u8 => N250.clone(),
            251u8 => N251.clone(),
            252u8 => N252.clone(),
            253u8 => N253.clone(),
            254u8 => N254.clone(),
            255u8 => N255.clone(),
            _ => panic!("too many bits for this u8")
        }
    }
}


impl<'a> UnsignedDCNumber<'a> {
    pub fn new<T>(digits: T, last_integer: usize) -> Self
        where
            Cow<'a, [u8]>: From<T>,
    {
        let v: Cow<[u8]> = digits.into();
        debug_assert!(last_integer <= v.len(), "separator {} should be less than {}: v{:?}", last_integer, v.len(), v);

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
    ///
    /// assert_eq!(UnsignedDCNumber::from_bytes_radix("0".as_ref(), 8), UnsignedDCNumber::from_byte_radix(b'0', 8));
    /// assert_eq!(UnsignedDCNumber::from_bytes_radix("0".as_ref(), 8), UnsignedDCNumber::from_byte_radix(b'0', 10));
    /// ```
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
        // implementing this "in place"
        eprint!("XXXXXX {:?} + {:?} => ", self, other);
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

        // TODO: we should make sure that numbers of different length are properly aligned

        let mut self_digits = self_digits.iter().cloned();
        let mut other_digits = other_digits.iter().cloned();

        match self_separator.cmp(&other_separator) {
            Ordering::Less => {
                for _ in 0..(other_separator - self_separator) {
                    sum_digits.push_back(other_digits.next().unwrap())
                }
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                for _ in 0..(self_separator - other_separator) {
                    sum_digits.push_back(self_digits.next().unwrap())
                }
            }
        }


        let mut carry = false;
        for (mut lhs, rhs) in self_digits.rev().zip(other_digits.rev())
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
        eprintln!("XXXXXX {:?} {}", sum_digits, separator);
        UnsignedDCNumber::new(Vec::from(sum_digits), separator)
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

    /// assert_eq!(UnsignedDCNumber::from_bytes("10".as_ref()), UnsignedDCNumber::from_byte_radix(b'A', 16));
    ///
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'A', 8));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'A', 10));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'.', 8));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'.', 10));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'.', 16));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'a', 8));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'a', 10));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b'a', 16));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b';', 8));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b';', 10));
    /// assert_eq!(Err(ParseDCNumberError::InvalidDigit), UnsignedDCNumber::from_byte_radix(b';', 16));
    /// assert_eq!(Err(ParseDCNumberError::InvalidRadix), UnsignedDCNumber::from_byte_radix(b'0', 0));
    /// assert_eq!(Err(ParseDCNumberError::InvalidRadix), UnsignedDCNumber::from_byte_radix(b'0', 1));
    /// assert_eq!(Err(ParseDCNumberError::InvalidRadix), UnsignedDCNumber::from_byte_radix(b'0', 17));
    ///

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
            assert_eq!(
                i.to_string(),
                String::from_utf8(out).expect("utf8 issue"),
            )
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
    //    from_bytes_radix![first_hex: 10 = A: 16];
    //    from_bytes_radix![b2_10: 2 = 10: 2];
    //    from_bytes_radix![b3_10: 3 = 10: 3];
    //    from_bytes_radix![b4_10: 4 = 10: 4];
    //    from_bytes_radix![b5_10: 5 = 10: 5];
    //    from_bytes_radix![b6_10: 6 = 10: 6];
    //    from_bytes_radix![b7_10: 7 = 10: 7];
    //    from_bytes_radix![b8_10: 8 = 10: 8];
    //    from_bytes_radix![b9_10: 9 = 10: 9];
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
