use num::{self, ToPrimitive};
use std::cmp::{max, Ordering};
use std::collections::VecDeque;
use std::f32;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::iter::Iterator;
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

use dcnumber::digits_type::{DigitsType};
use dcnumber::error::ParseDCNumberError;
use dcnumber::traits::FromBytes;




#[derive(Debug)]
pub struct UnsignedDCNumber {
    // TODO: maybe use nibble?
    // digits are in BigEndian
    digits: DigitsType,
    // also consider having a pool for these numbers for memory locality
    separator: usize,
}

macro_rules! static_unsigned_dcnumber {
    ($dcnumber_name:ident; $digits_name:ident : $digits_type:ty = $digits:expr) => {
        #[allow(dead_code)]
        const $digits_name: $digits_type = $digits;
        #[allow(dead_code)]
        static $dcnumber_name: UnsignedDCNumber = UnsignedDCNumber {
            digits: DigitsType::Ref(&$digits),
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

unsafe impl Sync for UnsignedDCNumber {

}


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

    static SMALL_INTS: [&UnsignedDCNumber; 256] = [
        &N0, &N1, &N2, &N3, &N4, &N5, &N6, &N7, &N8, &N9, &N10, &N11, &N12, &N13, &N14, &N15, &N16,
        &N17, &N18, &N19, &N20, &N21, &N22, &N23, &N24, &N25, &N26, &N27, &N28, &N29, &N30, &N31,
        &N32, &N33, &N34, &N35, &N36, &N37, &N38, &N39, &N40, &N41, &N42, &N43, &N44, &N45, &N46,
        &N47, &N48, &N49, &N50, &N51, &N52, &N53, &N54, &N55, &N56, &N57, &N58, &N59, &N60, &N61,
        &N62, &N63, &N64, &N65, &N66, &N67, &N68, &N69, &N70, &N71, &N72, &N73, &N74, &N75, &N76,
        &N77, &N78, &N79, &N80, &N81, &N82, &N83, &N84, &N85, &N86, &N87, &N88, &N89, &N90, &N91,
        &N92, &N93, &N94, &N95, &N96, &N97, &N98, &N99, &N100, &N101, &N102, &N103, &N104, &N105,
        &N106, &N107, &N108, &N109, &N110, &N111, &N112, &N113, &N114, &N115, &N116, &N117, &N118,
        &N119, &N120, &N121, &N122, &N123, &N124, &N125, &N126, &N127, &N128, &N129, &N130, &N131,
        &N132, &N133, &N134, &N135, &N136, &N137, &N138, &N139, &N140, &N141, &N142, &N143, &N144,
        &N145, &N146, &N147, &N148, &N149, &N150, &N151, &N152, &N153, &N154, &N155, &N156, &N157,
        &N158, &N159, &N160, &N161, &N162, &N163, &N164, &N165, &N166, &N167, &N168, &N169, &N170,
        &N171, &N172, &N173, &N174, &N175, &N176, &N177, &N178, &N179, &N180, &N181, &N182, &N183,
        &N184, &N185, &N186, &N187, &N188, &N189, &N190, &N191, &N192, &N193, &N194, &N195, &N196,
        &N197, &N198, &N199, &N200, &N201, &N202, &N203, &N204, &N205, &N206, &N207, &N208, &N209,
        &N210, &N211, &N212, &N213, &N214, &N215, &N216, &N217, &N218, &N219, &N220, &N221, &N222,
        &N223, &N224, &N225, &N226, &N227, &N228, &N229, &N230, &N231, &N232, &N233, &N234, &N235,
        &N236, &N237, &N238, &N239, &N240, &N241, &N242, &N243, &N244, &N245, &N246, &N247, &N248,
        &N249, &N250, &N251, &N252, &N253, &N254, &N255,
    ];

    #[inline(always)]
    pub fn interned(n: u8) -> UnsignedDCNumber {
        get_ref(n).dup()
    }

    #[inline(always)]
    pub fn get_ref(n: u8) -> &'static UnsignedDCNumber {
        unsafe {
            SMALL_INTS.get_unchecked(n as usize)
        }
    }

    #[inline(always)]
    pub fn zero_ref() -> &'static UnsignedDCNumber {
        get_ref(0)
    }

    #[inline(always)]
    pub fn one_ref() -> &'static UnsignedDCNumber {
        get_ref(1)
    }

    #[inline(always)]
    pub fn zero() -> UnsignedDCNumber {
        interned(0)
    }

    #[inline(always)]
    pub fn one() -> UnsignedDCNumber {
        interned(1)
    }
}

impl UnsignedDCNumber {
    pub fn new<T>(digits: T, last_integer: usize) -> Self
    where
        DigitsType: From<T>,
    {
        let v: DigitsType = digits.into();
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
        DigitsType: From<T>,
    {
        let digits: DigitsType = digits.into();
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

    fn cmp_unsigned(&self, other: &UnsignedDCNumber) -> Ordering {
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

    fn clone_into_parts(self) -> (Vec<u8>, usize) {
        let separator = self.separator;
        (self.digits.into_vec(), separator)
    }


    pub fn dup(&self) -> UnsignedDCNumber {
        UnsignedDCNumber{digits: self.digits.clone(), separator: self.separator}
    }

    fn inner_binop<F>(self, other: UnsignedDCNumber, f: F) -> Self
    where
        F: Fn(Vec<u8>, usize, &[u8], usize) -> Self
    {
        let UnsignedDCNumber { digits: self_digits, separator: self_separator } = self;

        if !self_digits.holds_memory() {
            if self_digits.len() < other.digits.len() {
                let UnsignedDCNumber{digits: other_digits, separator: other_separator} = other;
                let other_digits = other_digits.into_vec();
                return f(other_digits, other_separator, self_digits.as_ref(), self_separator);

            }
        }

        let self_digits = self_digits.into_vec();


        f(self_digits, self_separator, other.digits.as_ref(), other.separator)
    }

    fn inner_sub(self, other: UnsignedDCNumber) -> Self {
        self.inner_binop(other, inner_sub_digits_ref)
    }

    fn inner_add(self, other: UnsignedDCNumber) -> Self {
        let UnsignedDCNumber { digits: self_digits, separator: self_separator } = self;

        if !self_digits.holds_memory() {
            if self_digits.len() < other.digits.len() {
                let UnsignedDCNumber{digits: other_digits, separator: other_separator} = other;
                let other_digits = other_digits.into_vec();
                return inner_add_digits_ref(other_digits, other_separator, self_digits.as_ref(), self_separator);

            }
        }

        let self_digits = self_digits.into_vec();

        inner_add_digits_ref(self_digits, self_separator, other.digits.as_ref(), other.separator)
    }

    fn is_integer(&self) -> bool {
        self.digits.len() <= self.separator
    }

    fn _is_zero(&self) -> bool {
        self.separator == 1 && self.digits.len() == 1 && self.digits.first()
            .map(|&d| d == 0)
            .unwrap_or(false)
   }

    fn mul_10(self) -> Self {
        if self._is_zero() {
            return self;
        }
        let (mut v, mut separator) = self.clone_into_parts();
        if separator >= v.len() {
            // it is an integer
            // 1 => 10
            v.push(0);
            separator += 1;
        } else {
            // this is a fractional number
            // 0.12 => 1.2
            // 0.01 => 0.1
            // 0.1 => 1
            // 1.1 => 11
            if v[0] == 0 {
                let mut dq = VecDeque::from(v);
                let _ = dq.pop_front();
                v = Vec::from(dq);
            } else {
                separator += 1;
            }
        }
        UnsignedDCNumber { digits: DigitsType::from(v), separator }
    }

}

#[inline(always)]
fn align_dcunsigned(lhs: &[u8], lhs_separator: usize, rhs: &[u8], rhs_separator: usize) -> (usize, usize, usize, usize, usize, usize) {
    let lhs_fractional_digits = lhs.len() - lhs_separator;
    let rhs_fractional_digits = rhs.len() - rhs_separator;

    let lhs_aligned_index;
    let rhs_aligned_index;

    if lhs_fractional_digits > rhs_fractional_digits {
        let fractional_offset = lhs_fractional_digits - rhs_fractional_digits;
        lhs_aligned_index = (lhs.len() - fractional_offset) - 1;
        rhs_aligned_index = rhs.len() - 1;
    } else {
        let fractional_offset = rhs_fractional_digits - lhs_fractional_digits;
        rhs_aligned_index = (rhs.len() - fractional_offset) - 1;
        lhs_aligned_index = lhs.len() - 1;
    }

    let lhs_aligned_end;
    let rhs_aligned_end;
    if lhs_separator > rhs_separator {
        lhs_aligned_end = lhs_separator - rhs_separator;
        rhs_aligned_end = 0;
    } else {
        rhs_aligned_end = rhs_separator - lhs_separator;
        lhs_aligned_end = 0;
    }
    (lhs_fractional_digits, rhs_fractional_digits, lhs_aligned_index, rhs_aligned_index, lhs_aligned_end, rhs_aligned_end)
}

#[inline(always)]
fn inner_add_digits_ref<'b>(mut lhs: Vec<u8>, lhs_separator: usize, rhs: &'b [u8], rhs_separator: usize) -> UnsignedDCNumber {
    let mut carry = false;
    let mut separator = max(lhs_separator, rhs_separator);

    let (
        lhs_fractional_digits, rhs_fractional_digits, lhs_aligned_index,
        rhs_aligned_index, lhs_aligned_end, rhs_aligned_end
    ) = align_dcunsigned(lhs.as_ref(), lhs_separator, rhs, rhs_separator);

    lhs[lhs_aligned_end..lhs_aligned_index+1].iter_mut().rev()
        .zip(rhs[rhs_aligned_end..rhs_aligned_index+1].iter().rev())
        .for_each(|(lhs, &rhs)| {
            debug_assert!(*lhs < 10, "{} < 10", lhs);
            debug_assert!(rhs < 10);

            if carry {
                *lhs += rhs + 1;
                carry = false;
            } else {
                *lhs += rhs;
            }
            debug_assert!(*lhs < 19, "{} < 19", lhs);

            if *lhs >= 10 {
                *lhs -= 10;
                carry = true;
            }

            debug_assert!(*lhs < 10, "{} < 10", lhs);
        });

    lhs[0..lhs_aligned_end].iter_mut()
        .rev()
        .for_each(|lhs| {
            debug_assert!(*lhs < 10);
            if carry {
                if *lhs == 9 {
                    *lhs = 0;
                } else {
                    *lhs += 1;
                    carry = false;
                }

            }
            debug_assert!(*lhs < 10);
        });

    if rhs_aligned_end > 0 {
        lhs.extend(rhs[0..rhs_aligned_end].iter()
            .cloned()
            .rev()
            .map(|d| {
                let sum = if carry {
                    carry = false;
                    d + 1
                } else { d };
                if sum >= 10 {
                    carry = true;
                    sum - 10
                } else {
                    sum
                }
            })
        );
        lhs.rotate_right(rhs_aligned_end);
    }

    if carry {
        lhs.insert(0, 1);
        separator += 1;
    }

    if rhs_fractional_digits > lhs_fractional_digits {
        lhs.extend(&rhs[rhs.len() - (rhs_fractional_digits - lhs_fractional_digits)..rhs.len()]);
    }

    UnsignedDCNumber::new(lhs, separator)
}

#[inline(always)]
fn inner_sub_digits_ref<'b>(mut lhs: Vec<u8>, lhs_separator: usize, rhs: &'b [u8], rhs_separator: usize) -> UnsignedDCNumber {
    let mut carry = false;
    let separator = max(lhs_separator, rhs_separator);

    let (
        lhs_fractional_digits, rhs_fractional_digits, lhs_aligned_index,
        rhs_aligned_index, lhs_aligned_end, rhs_aligned_end
    ) = align_dcunsigned(lhs.as_ref(), lhs_separator, rhs, rhs_separator);

    debug_assert!(rhs_aligned_end == 0, "lhs should be > than rhs");

    // First handle the smallest fractional digits
    // A. if lhs has more fractional digits, there is nothing to do
    // B. if rhs has more fractional digits, we need to handle it as if
    //    lhs was padded with 0s
    let additional_fractional_digits = rhs_fractional_digits.saturating_sub(lhs_fractional_digits);
    lhs.reserve(additional_fractional_digits);

    lhs.extend(
        rhs[rhs_aligned_index+1..].iter().rev()
            .map(|&rhs| {
                // TODO a faster way to write this is consider
                // that only for the "last" (largest index) digit we have no carry
                // if additional_fractional_digits != 0
                if carry {
                    carry = true;
                    9 - rhs
                } else {
                    carry = true;
                    10 - rhs
                }
            }).rev());

    lhs[lhs_aligned_end..lhs_aligned_index+1].iter_mut().rev()
        .zip(rhs[rhs_aligned_end..rhs_aligned_index+1].iter().rev())
        .for_each(|(lhs, &rhs)| {
            let wrapped_around = lhs.wrapping_sub(if carry {
                carry = false;
                rhs+1
            } else {rhs});
            if wrapped_around > 10 {
                *lhs = wrapped_around.wrapping_add(10);
                carry = true;
            } else {
                *lhs = wrapped_around;
            }
        });

    if carry {
        for lhs in lhs[..lhs_aligned_end].iter_mut().rev() {
            if !carry { break; }
            *lhs = lhs.wrapping_sub(1);
            if *lhs > 10 {
                *lhs = lhs.wrapping_add(10);
                carry = true;
            }
        }

        if let Some(to_remove) = lhs.iter().position(|&d| d != 0) {
            let len = lhs.len();
            lhs.rotate_left(to_remove );
            lhs.truncate(len - to_remove);
            return UnsignedDCNumber::new(lhs, separator - to_remove);
        };
    }

    return UnsignedDCNumber::new(lhs, separator);
}

impl Default for UnsignedDCNumber {
    fn default() -> Self {
        small_ints::zero()
    }
}

impl PartialEq for UnsignedDCNumber {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_unsigned(other) == Ordering::Equal
    }
}


impl Eq for UnsignedDCNumber {}

impl PartialOrd for UnsignedDCNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_unsigned(other))
    }
}

impl Ord for UnsignedDCNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_unsigned(other)
    }
}

// TODO add similar to test_partial_order for cmp as well

impl ToPrimitive for UnsignedDCNumber {
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

impl Display for UnsignedDCNumber {
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


macro_rules! lsd {
    ($n:expr) => {
        ($n % 10) as u8
    };
}


// TODO consiider implementing *= u8; migght be the fastest option here (MulAssign)

impl Mul<UnsignedDCNumber> for UnsignedDCNumber {
    type Output = UnsignedDCNumber;

    fn mul(self, _rhs: UnsignedDCNumber) -> Self {
        unimplemented!()
    }
}

impl Sub<UnsignedDCNumber> for UnsignedDCNumber {
    type Output = UnsignedDCNumber;


    fn sub(self, rhs: UnsignedDCNumber) -> Self {
        self.inner_sub(rhs)
    }
}

impl Add<UnsignedDCNumber> for UnsignedDCNumber {
    type Output = UnsignedDCNumber;

    fn add(self, other: UnsignedDCNumber) -> Self {
        self.inner_add(other)
    }
}


// TODO consider if implementing Add<&> allows us faster

impl num::Zero for UnsignedDCNumber {
    fn zero() -> Self {
        small_ints::zero()
    }

    fn is_zero(&self) -> bool {
        if self == small_ints::zero_ref() {
            true
        } else {
            false
        }
    }
}

impl num::One for UnsignedDCNumber {
    fn one() -> Self {
        small_ints::one()
    }

    fn is_one(&self) -> bool where Self: PartialEq {
        if self == small_ints::one_ref() {
            true
        } else {
            false
        }
    }
}

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

macro_rules! impl_from_unsigned_primitive_u8 {
    ($u:ty) => {
    };
}

macro_rules! impl_from_unsigned_primitive {
    ($u:ty) => {
        impl From<$u> for UnsignedDCNumber {
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

        impl Add<$u> for UnsignedDCNumber {
            type Output = UnsignedDCNumber;

            fn add(self, other: $u) -> Self::Output {
                // TODO make this more efficient by implementing Add "in place"
                self + UnsignedDCNumber::from(other)
            }
        }

        impl_from_unsigned_primitive_u8!($u);
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
impl From<u8> for UnsignedDCNumber {
    fn from(n: u8) -> Self {
        small_ints::interned(n)
    }
}

impl Add<u8> for UnsignedDCNumber {
    type Output = UnsignedDCNumber;

    fn add(self, other: u8) -> Self::Output {
        self + small_ints::interned(other)
    }
}


impl Mul<u8> for UnsignedDCNumber {
    type Output = UnsignedDCNumber;

    fn mul(self, other: u8) -> Self::Output {
        // TODO put us back
        // optimize 0, 1, 10, 100
//        if self.is_zero() {
//            return self;
//        }
//
//        if self.is_one() {
//            return UnsignedDCNumber::from(other);
//        }

        if other == 0 {
            return small_ints::zero();
        }

        if other == 1 {
            return self;
        }

        if other == 10 {
            return self.mul_10();
        }

        let (v, separator) = self.clone_into_parts();
        let mut separator = separator;


        // TODO: we can do this on demand only if we need to
        let mut digits  : VecDeque<u8> = VecDeque::from(v);

        // TODO try with different values here e.g., u32
        type MulT = u16;
        let mut global_result: MulT = 0;

        // 1881 = 19 * 99];

        digits.iter_mut().rev()
            .for_each(|d| {
                global_result += *d as MulT * other as MulT;
                *d = lsd!(global_result);
                global_result /= 10;
            });

        // if we had "overflow" for this digit, we should create the right
        while global_result > 0 {
            digits.push_front(lsd!(global_result));
            separator += 1;
            global_result /= 10;
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

impl_from_unsigned_primitive_u8![u8];



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
    use super::{ParseDCNumberError, UnsignedDCNumber, small_ints};

    pub trait AsciiConverter {
        fn convert_bytes(
            &self,
            bytes: &[u8],
        ) -> Result<UnsignedDCNumber, ParseDCNumberError>;
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
        fn convert_bytes(
            &self,
            bytes: &[u8],
        ) -> Result<UnsignedDCNumber, ParseDCNumberError> {
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
        fn convert_bytes(
            &self,
            bytes: &[u8],
        ) -> Result<UnsignedDCNumber, ParseDCNumberError> {
            let radix = self.radix;
            bytes.iter().fold(Ok(small_ints::zero()), |acc, &ch| {
                acc.and_then(|n| {
                    UnsignedDCNumber::from_byte_radix_u8(ch, radix)
                        .and_then(|m| Ok(m + (n * radix)))
                })
            })
        }
    }
}

impl FromBytes for UnsignedDCNumber {
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
        use self::radix_converters::AsciiConverter;
        radix_converters::DecAsciiConverter::new().convert_bytes(bytes)
    }

}

impl FromStr for UnsignedDCNumber {
    type Err = ParseDCNumberError;

    fn from_str(s: &str) -> Result<Self, ParseDCNumberError> {
        FromBytes::from_bytes(s.as_ref())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! zero_literal {
        () => (UnsignedDCNumber{digits: digits![0], separator: 1});
    }

    macro_rules! one_literal {
        () => (UnsignedDCNumber{digits: digits![1], separator: 1});
    }

    #[test]
    fn test_is_zero() {
        let zero = UnsignedDCNumber { digits: digits!(0), separator: 1 };
        assert!(zero._is_zero());
        let one = UnsignedDCNumber { digits: digits!(1), separator: 1 };
        assert!(!one._is_zero());
        let zero_dot = UnsignedDCNumber { digits: digits!(0, 1), separator: 1 };
        assert!(!zero_dot._is_zero());
    }

    #[test]
    fn test_default() {
        assert_eq!(UnsignedDCNumber { digits: digits![0], separator: 1 }, UnsignedDCNumber::default());
    }

    #[test]
    fn test_split() {
        assert_eq!(([0u8].as_ref(), [].as_ref()), small_ints::zero().split());
    }

    #[test]
    fn test_split1() {
        assert_eq!(([1u8].as_ref(), [].as_ref()), one_literal!().split());
    }

    #[test]
    fn test_split2() {
        assert_eq!(
            ([1, 2, 3, 4].as_ref(), [3, 2].as_ref()),
            udcn!("1234.32").split()
        );
    }

    #[test]
    fn test_split3() {
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
                fn str_bytes2() {
                    assert_eq!( UnsignedDCNumber::from_str(stringify!($digits).as_ref())
                            .expect(stringify!($digits)),
                        FromBytes::from_bytes(stringify!($digits).as_ref())
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
            ($test_name:ident : $digits:tt; 10) => {
                mod $test_name {
                    use super::*;

                    #[test]
                    fn from_byte() {
                        assert_eq!(
                            UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), 10),
                            UnsignedDCNumber::from_byte(stringify!($digits).as_bytes()[0])
                        )
                    }

                    #[test]
                    fn from_byte_radix() {
                        assert_eq!(
                            UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), 10),
                            UnsignedDCNumber::from_byte_radix(stringify!($digits).as_bytes()[0], 10)
                        )
                    }

                }

        };
        ($test_name:ident : $digits:tt; $radix:expr) => {

            #[test]
            fn $test_name() {
                assert_eq!(
                    UnsignedDCNumber::from_bytes_radix(stringify!($digits).as_ref(), $radix),
                    UnsignedDCNumber::from_byte_radix(stringify!($digits).as_bytes()[0], $radix)
                )
            }
        };
        ($test_name:ident : $lhs_digits:expr, $lhs_radix:expr; $rhs_digits:expr) => {
            mod $test_name {
                use super::*;

                #[test]
                fn from_byte() {
                    assert_eq!(
                        UnsignedDCNumber::from_byte_radix($lhs_digits, $lhs_radix),
                        UnsignedDCNumber::from_byte($rhs_digits, 10),
                    );
                }

                #[test]
                fn from_byte_radix() {
                    assert_eq!(
                        UnsignedDCNumber::from_byte_radix($lhs_digits, $lhs_radix),
                        UnsignedDCNumber::from_byte_radix($rhs_digits, 10),
                    );
                }
            }
        };
        ($test_name:ident : $lhs_digits:expr, $lhs_radix:expr; $rhs_digits:expr, $rhs_radix:expr) => {
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
    test_from_byte_radix!(from_byte_radix_a_8: A; 8);
    test_from_byte_radix!(from_byte_radix_a_10: A; 10);
    test_from_byte_radix!(from_byte_radix_a_16: A; 16);

    #[test]
    fn test_regression_a_16() {
        let n = UnsignedDCNumber::from_str_radix("A", 16).expect("A in hex should be fine");
        assert_eq!(UnsignedDCNumber::new(digits![1, 0], 2), n);
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
        assert_eq!(Ordering::Equal, zero_literal!().cmp_unsigned(&zero_literal!()));
        assert_eq!(Ordering::Less, zero_literal!().cmp_unsigned(&one_literal!()));
        assert_eq!(Ordering::Greater, one_literal!().cmp_unsigned(&zero_literal!()));
        assert_eq!(Ordering::Equal, one_literal!().cmp_unsigned(&one_literal!()));
    }

    #[test]
    fn test_eq() {
        assert_eq!(zero_literal!(), zero_literal!());
        assert_eq!(one_literal!(), one_literal!())
    }

    #[test]
    fn test_partial_order() {
        assert_eq!(Some(Ordering::Less), zero_literal!().partial_cmp(&one_literal!()));
        assert_eq!(Some(Ordering::Greater), one_literal!().partial_cmp(&zero_literal!()));
        assert_eq!(Some(Ordering::Equal), zero_literal!().partial_cmp(&zero_literal!()));
        assert_eq!(
            Some(Ordering::Less),
            UnsignedDCNumber::from(213 as u32)
                .partial_cmp(&UnsignedDCNumber::from_str("321.12").unwrap())
        );
    }

    #[test]
    fn test_order() {
        assert!(zero_literal!() < one_literal!());
    }

    #[test]
    fn test_to_primitive() {
        assert_eq!(0, zero_literal!().to_u64().expect("u64 zero_literal!()"));
        assert_eq!(1, one_literal!().to_u64().expect("u64 one"));
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

        assert_eq!(0, zero_literal!().to_i64().expect("i64 zero"));
        assert_eq!(1, one_literal!().to_i64().expect("i64 one"));
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
                let expected = udcn![stringify!($expected)];
                let lhs = udcn![stringify!($lhs)];
                let rhs = udcn![stringify!($rhs)];

                assert_eq!(expected, lhs.dup() $op rhs.dup());
                assert_eq!(expected, lhs.dup() $op udcn![stringify!($rhs)]);
                assert_eq!(expected, udcn![stringify!($lhs)] $op rhs.dup());

                assert_eq!(expected, lhs $op rhs);
            }
        };
       (borrowed $test_name:ident : $expected:tt = $lhs:tt $op:tt $rhs:tt) => {
            #[test]
            fn $test_name() {
                use super::small_ints;
                let expected = udcn![stringify!($expected)];
                let lhs: UnsignedDCNumber = small_ints::interned($lhs);
                let rhs: UnsignedDCNumber = small_ints::interned($rhs);

                assert_eq!(expected, lhs.dup() $op rhs.dup(), "dup dup");
//                assert_eq!(expected, lhs.dup() $op  small_ints::interned($lhs), "dup -");
//                assert_eq!(expected, small_ints::interned($rhs) $op rhs.dup(), "- dup");

                assert_eq!(expected, lhs $op rhs, "- -");
            }
        };
        (u8 $test_name:ident : $expected:tt = $lhs:tt $op:tt $rhs:expr) => {
            #[test]
            fn $test_name() {
                assert_eq!(
                    udcn![stringify!($expected)],
                    udcn![stringify!($lhs)] $op $rhs as u8,
                );
            }
        };
    }

    test_binop![add_zero: 0 = 0 + 0];
    test_binop![add_unit: 1 = 1 + 0];
    test_binop![add_two: 2 = 2 + 0];
    test_binop![add_unit2: 1 = 1 + 0];
    test_binop![add_units: 2 = 1 + 1];
    test_binop![add_to_three: 10 = 3 + 7];
    test_binop![add_seven: 7 = 0 + 7];
    test_binop![integers: 1026 = 520 + 506];
    test_binop![add_frac: 20.2 = 10.1 + 10.1];
    test_binop![add_f:10143.043 = 7221.123 + 2921.92];
    test_binop![add_f1:20143.043 = 17221.123 + 2921.92];
    test_binop![add_f2:20143.043 = 7221.123 + 12921.92];
    test_binop![add_f3b:110 = 101 + 9];
    test_binop![add_f3c:112 = 103 + 9];
    test_binop![add_f3:110143.043 = 107221.123 + 2921.92];
    test_binop![add_f4:110143.043 = 7221.123 + 102921.92];
    test_binop![add_le:10.1 = 9.9 + 0.2];
    test_binop![add_le2:10.12 = 9.9 + 0.22];
    test_binop![add_le3:10.12 = 9.92 + 0.2];
    test_binop![add_le4:10.12 = 0.92 + 9.2];
    test_binop![add_le5:1000.12 = 990.92 + 9.2];
    test_binop![add_le6:1000.12 = 999.92 + 0.2];
    test_binop![add_le7:1000.12 = 9.2 + 990.92];
    test_binop![add_le8:1000.12 = 0.2 + 999.92];
    test_binop![u8 add_zero_u8: 0 = 0 + 0];
    test_binop![u8 add_unit_u8: 1 = 0 + 1];

    test_binop![borrowed badd_zero: 0 = 0 + 0];
    test_binop![borrowed badd_unit: 1 = 1 + 0];
    test_binop![borrowed badd_unit2: 1 = 0 + 1];
    test_binop![borrowed bborroweds: 254 = 127 + 127 ];

    test_binop![sub_zero: 0 = 0 - 0];
    test_binop![simple_9_3: 6 = 9 - 3];
    test_binop![simple_4_4: 0 = 4 - 4];
    test_binop![int_carry_10_7: 3 = 10 - 7];
    test_binop![int_carry_100_3: 97 = 100 - 3];
    test_binop![int_carry_100_30: 70 = 100 - 30];
    test_binop![int_carry_200_30: 170 = 200 - 30];
    test_binop![frac_simple_9_3_0: 6.3 = 9.3 - 3.0];
    test_binop![frac_simple_9_3: 6.3 = 9.3 - 3];
    test_binop![frac_carry_10_dot7: 9.3 = 10 - 0.7];
    test_binop![frac_carry_100_dot3: 99.7 = 100 - 0.3];
    test_binop![frac_carry_100_dot30: 99.7 = 100 - 0.30];
    test_binop![frac_carry_103dot23_1dot1: 102.13 = 103.23 - 1.1 ];

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
        assert_eq!(zero_literal!(), zero);
    }

    #[test]
    fn test_from_u64_10() {
        let n = UnsignedDCNumber::from(10 as u64);
        assert_eq!(UnsignedDCNumber::new(digits![1, 0], 2), n);
    }

    #[test]
    fn test_from_u64_one() {
        let one = UnsignedDCNumber::from(1 as u64);
        assert_eq!(one_literal!(), one);
    }

    #[test]
    fn test_from_u64() {
        let n = UnsignedDCNumber::from(1234567890 as u64);
        assert_eq!(
            UnsignedDCNumber::with_integer_digits(digits![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]),
            n
        );
    }

    macro_rules! test_from_bytes {
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
                fn from_bytes2() {
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
                fn from_bytes2() {
                    assert_eq!(
                        $expected,
                        FromBytes::from_bytes(stringify!($digits).as_ref())
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
                fn test_from_bytes_radix_16(b: &mut Bencher) {
                    b.iter(|| {
                        UnsignedDCNumber::from_bytes_radix($digits.as_ref(), 16)
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


    test_from_bytes![from_str_zero: zero_literal!() ; 0];
    test_from_bytes![from_str_one:  one_literal!() ; 1];
    test_from_bytes![from_str_10: UnsignedDCNumber::new(digits![1, 0], 2) ; 10];
    test_from_bytes![from_str_byte_spec: UnsignedDCNumber::new(digits![1, 1], 1) ; 1.1];
    test_from_bytes![from_str_0dot9: UnsignedDCNumber::new(digits![0, 9], 1) ; 0.9];
    test_from_bytes![from_str_1000dot3: UnsignedDCNumber::new(digits![1, 0, 0, 0, 3], 4) ; 1000.3];
    test_from_bytes![from_str_0dot01: UnsignedDCNumber::new(digits![0, 0, 1], 1) ; 0.01];
    test_from_bytes![from_str_from_int: UnsignedDCNumber::from(1234 as u16) ; 1234 ];
    test_from_bytes![from_str_from_int_leading0: UnsignedDCNumber::from(1234 as u16) ; 01234];
    test_from_bytes![from_str_empty : EmptyString <- ""];
    test_from_bytes![from_str_a : InvalidDigit <- "a"];
    test_from_bytes![from_str_1a : InvalidDigit <- "1a]"];
    test_from_bytes![from_str_0a : InvalidDigit <- "0a"];
    test_from_bytes![from_str_dota : InvalidDigit <- ".a"];
    test_from_bytes![from_str_0dotdot0: RepeatedDot <- "0..0"];
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

    #[test]
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
    from_bytes_radix![b2_10: 2 = 10: 2];
    from_bytes_radix![b3_10: 3 = 10: 3];
    from_bytes_radix![b4_10: 4 = 10: 4];
    from_bytes_radix![b5_10: 5 = 10: 5];
    from_bytes_radix![b6_10: 6 = 10: 6];
    from_bytes_radix![b7_10: 7 = 10: 7];
    from_bytes_radix![b8_10: 8 = 10: 8];
    from_bytes_radix![b9_10: 9 = 10: 9];
    from_bytes_radix![b8_0: 0 = 0: 8];
    from_bytes_radix![b8_1: 1 = 1: 8];
    from_bytes_radix![b8_2: 2 = 2: 8];
    from_bytes_radix![b10_0: 0 = 0: 10];
    from_bytes_radix![b10_1: 1 = 1: 10];
    from_bytes_radix![b10_2: 2 = 2: 10];
    from_bytes_radix![b16_0: 0 = 0: 16];
    from_bytes_radix![b16_1: 1 = 1: 16];
    from_bytes_radix![b16_2: 2 = 2: 16];
    from_bytes_radix![b16_a: 10 = A: 16];

    bench_from_str![short_int: "3"];
    bench_from_str![mid_int: "17235428"];
    bench_from_str![long_int: "172354283422734622371431236441234351267438543781453193415694871634731457681354784531"];
    bench_from_str![longer_int: "17235428342273462237143123644123435126743854378145319341569487000000000000163473145768135478453123187356412946123041213310238698752341280000000000000000000000"];


    #[cfg(all(feature = "nightly", test))]
    mod benches {
        use super::*;
        use test::{Bencher, black_box};



        macro_rules! bench_on_integer_inner {
            ($bench_name:ident: $n:expr; $f:expr) => {
                #[bench]
                fn $bench_name(b: &mut Bencher) {
                    let n = UnsignedDCNumber::from_str($n).unwrap();

                    b.iter(||{
                        $f(&n)
                    })
                }
            };
        }

        macro_rules! bench_on_integer {
            ($bench_name:ident : $n: expr) => {
                mod $bench_name {
                    use super::*;

                    bench_on_integer_inner!(split: $n; |n: &UnsignedDCNumber| {
                        black_box(n.split());
                    });

                    bench_on_integer_inner!(dup: $n; |n: &UnsignedDCNumber| {
                        black_box(n.dup());
                    });
                    bench_on_integer_inner!(fmt: $n; |n: &UnsignedDCNumber| {
                        use std::io::Write;

                        let mut buf = Vec::new();
                        black_box(write!(buf, "{}", n).unwrap());
                    });
                }
            }
        }

        bench_on_integer!(large: "341162541237485134671351132634154364513467314531843114671354132645132412349123415348314");
        bench_on_integer!(larger:
            concat!(
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314",
                "6731453184311467135413264513241234912341534531843114671354132645132412349123415348314"
            )
         );



        macro_rules! bench_intern {
            ($bench_name:ident : $n: expr) => {
                #[bench]
                fn $bench_name(b: &mut Bencher) {
                    b.iter(|| {
                        black_box(small_ints::interned($n));
                    });
                }

            }
        }

        bench_intern!(intern_0: 0);
        bench_intern!(intern_10: 10);
        bench_intern!(intern_100: 100);



        macro_rules! bench_new {
            ($bench_name:ident : $n: expr) => {
                mod $bench_name {
                    use super::*;

                    #[bench]
                    fn as_ref(b: &mut Bencher) {
                        b.iter(|| {
                            black_box(UnsignedDCNumber::new($n.as_ref(), 0))
                        });
                    }

                    #[bench]
                    fn vec(b: &mut Bencher) {
                        let mut v = $n.to_vec();

                        b.iter(|| {
                            let mut v2: Vec<u8> = Vec::new();
                            ::std::mem::swap(&mut v, &mut v2);
                            black_box(UnsignedDCNumber::new(v2, 0))
                        });
                    }

                    #[bench]
                    fn vec_create_inside_loop(b: &mut Bencher) {

                        b.iter(|| {
                            let v = $n.to_vec();
                            black_box(UnsignedDCNumber::new(v, 0))
                        });
                    }
                }
            }
        }


        bench_new!(new_121345678901: [
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
        ]);
        bench_new!(new_121345678901_more: [
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
        ]);
        bench_new!(new_121345678901_even_more: [
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1,
        ]);

        macro_rules! udcnd {
            ($digits:expr, $separator:expr) => {
                UnsignedDCNumber{
                    digits: DigitsType::from($digits.as_ref()),
                    separator: $separator,
                }
            }
        }

        macro_rules! bench_binops_internal {
            ($op_name: ident: $lhs: expr, $lhs_sep:expr, $rhs: expr, $rhs_sep: expr , $op:tt) => {
               #[bench]
                fn $op_name(b: &mut Bencher) {
                    b.iter(|| {
                        let u = udcnd!($lhs, $lhs_sep);
                        let v = udcnd!($rhs, $rhs_sep);

                        black_box(u $op v)
                    });
                }
            }
        }


        macro_rules! bench_binops_int {
            ($op_name: ident: $lhs: expr, $lhs_sep:expr, $n: expr, $op:tt) => {
               #[bench]
                fn $op_name(b: &mut Bencher) {
                    b.iter(|| {
                        let u = udcnd!($lhs, $lhs_sep);
                        black_box(u $op $n);
                    });
                }
            }
        }


        macro_rules! bench_binops {
            ($bench_name:ident : $lhs: expr, $lhs_sep:expr, $rhs: expr, $rhs_sep: expr) => {
                mod $bench_name {
                    use super::*;

                    bench_binops_internal!(add: $lhs, $lhs_sep, $rhs, $rhs_sep, +);
                    bench_binops_internal!(lt: $lhs, $lhs_sep, $rhs, $rhs_sep, <);
                    bench_binops_internal!(gt: $lhs, $lhs_sep, $rhs, $rhs_sep, >);
                    bench_binops_internal!(eq: $lhs, $lhs_sep, $rhs, $rhs_sep, ==);

                    bench_binops_int!(mul8_0: $lhs, $lhs_sep, 0, *);
                    bench_binops_int!(mul8_1: $lhs, $lhs_sep, 1, *);
                    bench_binops_int!(mul8_10: $lhs, $lhs_sep, 10, *);
                    bench_binops_int!(mul8_100: $lhs, $lhs_sep, 100, *);
                    bench_binops_int!(mul8_11: $lhs, $lhs_sep, 11, *);
                    bench_binops_int!(mul8_111: $lhs, $lhs_sep, 111, *);
                    bench_binops_int!(mul8_3: $lhs, $lhs_sep, 3, *);
                }


            }
        }

        bench_binops!(op_large:
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            ], 10, [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 5, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            ], 10
        );

    }

    macro_rules! from_primitive_int {
        ($test_name:ident : $digits:tt, $tp:tt) => {
            #[test]
            fn $test_name() {
                assert_eq!(UnsignedDCNumber::from_str(stringify!($digits)).unwrap(), UnsignedDCNumber::from($digits as $tp));
            }
        }
    }

    from_primitive_int!(test_0_u8: 0, u8);
    from_primitive_int!(test_1_u8: 1, u8);
    from_primitive_int!(test_2_u8: 2, u8);
    from_primitive_int!(test_10_u8: 10, u8);
    from_primitive_int!(test_12u8: 12, u8);
    from_primitive_int!(test_123u8: 123, u8);
    from_primitive_int!(test_223u8: 223, u8);
    from_primitive_int!(test_255u8: 255, u8);

    from_primitive_int!(test_0_u16: 0, u16);
    from_primitive_int!(test_1_u16: 1, u16);
    from_primitive_int!(test_2_u16: 2, u16);
    from_primitive_int!(test_10_u16: 10, u16);
    from_primitive_int!(test_12u16: 12, u16);
    from_primitive_int!(test_123u16: 123, u16);
    from_primitive_int!(test_223u16: 223, u16);
    from_primitive_int!(test_255u16: 255, u16);
    from_primitive_int!(test_256u32: 256, u32);
    from_primitive_int!(test_10256u32: 10256, u32);

    from_primitive_int!(test_0_u32: 0, u32);
    from_primitive_int!(test_1_u32: 1, u32);
    from_primitive_int!(test_2_u32: 2, u32);
    from_primitive_int!(test_10_u32: 10, u32);
    from_primitive_int!(test_12u32: 12, u32);
    from_primitive_int!(test_123u32: 123, u32);
    from_primitive_int!(test_223u32: 223, u32);
    from_primitive_int!(test_255u32: 255, u32);

    from_primitive_int!(test_0_u64: 0, u64);
    from_primitive_int!(test_1_u64: 1, u64);
    from_primitive_int!(test_2_u64: 2, u64);
    from_primitive_int!(test_10_u64: 10, u64);
    from_primitive_int!(test_12u64: 12, u64);
    from_primitive_int!(test_123u64: 123, u64);
    from_primitive_int!(test_223u64: 223, u64);
    from_primitive_int!(test_255u64: 255, u64);

    #[test]
    fn test_zero_and_one() {
        use num::Zero;
        use num::One;

        assert_eq!(zero_literal!(), UnsignedDCNumber::zero());
        assert_eq!(one_literal!(), UnsignedDCNumber::one());
        assert!(UnsignedDCNumber::zero().is_zero());
        assert!(!UnsignedDCNumber::one().is_zero());
        assert!(!UnsignedDCNumber::zero().is_one());
        assert!(UnsignedDCNumber::one().is_one());
    }
}

