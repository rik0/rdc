extern crate rdc;

use std::ffi::OsStr;
use std::io;
use std::process;

use std::process::Command;

fn run_dc<I, S>(args: I) -> io::Result<process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr> + From<&'static str>,
{
    let dc_args = prepare_arguments(args);

    let mut command = Command::new("dc");
    let c = (&mut command)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .args(dc_args.iter());

    let child = c.spawn()?;

    child.wait_with_output()
}

fn prepare_arguments<'a, I, S>(args: I) -> Vec<S>
where
    I: IntoIterator<Item = S>,
    S: From<&'static str>,
    // <S as std::str::FromStr>::Err: std::fmt::Debug,
{
    let mut dc_args = Vec::<S>::new();
    for arg in args.into_iter() {
        dc_args.push(S::from("-e"));
        dc_args.push(arg);
    }
    dc_args
}

macro_rules! test_dc {
    ($name:ident; $program:expr) => {
        #[test]
        #[allow(non_snake_case)]
        fn $name() {
            let stdout: Vec<u8> = Vec::new();
            let stderr: Vec<u8> = Vec::new();
            let programs = vec![$program];
            let expected = run_dc(programs.clone()).expect("process error");
            let dc_args = prepare_arguments(programs);
            let (actual_output, actual_error) = rdc::dc(dc_args.into_iter(), stderr, stdout);
            assert_eq!(
                (
                    String::from_utf8(expected.stdout).expect("utf error in system dc output"),
                    String::from_utf8(expected.stderr).expect("utf error in system dc stderr")
                ),
                (
                    String::from_utf8(actual_output).expect("utf8 output"),
                    String::from_utf8(actual_error).expect("utf8 error")
                ),
            );
        }
    };
}

#[test]
fn test() {
    let stdout: Vec<u8> = Vec::new();
    let stderr: Vec<u8> = Vec::new();
    let programs = vec!["10p"];
    let expected = run_dc(programs.clone()).expect("process error");
    let dc_args = prepare_arguments(programs);
    let (actual_output, actual_error) = rdc::dc(dc_args.into_iter(), stderr, stdout);
    assert_eq!(
        (
            String::from_utf8(expected.stdout).expect("utf error in system dc output"),
            String::from_utf8(expected.stderr).expect("utf error in system dc stderr")
        ),
        (
            String::from_utf8(actual_output).expect("utf8 output"),
            String::from_utf8(actual_error).expect("utf8 error")
        ),
    );
}

#[test]
#[allow(non_snake_case)]
fn test_huge_Q() {
    let stdout: Vec<u8> = Vec::new();
    let stderr: Vec<u8> = Vec::new();
    let programs = vec!["371946139746397463926439726439764969639436932476233984734843946937638974648736487643827 Q 10p"];
    let expected = run_dc(programs.clone()).expect("process error");
    let dc_args = prepare_arguments(programs);
    let (actual_output, actual_error) = rdc::dc(dc_args.into_iter(), stderr, stdout);
    assert_eq!(
            String::from_utf8(expected.stdout).expect("utf error in system dc output"),
            String::from_utf8(actual_output).expect("utf8 output"),
    );
    // under some systems, such large numbers 
    assert!(!actual_error.is_empty());
    assert!(expected.stderr.ends_with(&actual_error))
}

test_dc![_10p; "10p"];
test_dc![add; "10 20 + p"];
test_dc![sub; "10 20 - p"];
test_dc![mul; "10 20 * p"];
test_dc![div; "10 20 / p"];
test_dc![mod_; "10 20 % p"];

test_dc![empty_string;"[]zf"];

test_dc![dup;"10df"];
test_dc![dup_empty;"df"];

test_dc![swap_empty;"r"];
test_dc![swap_one;"10rf"];
test_dc![swap;"10 20 rf"];

test_dc![clear;"10cf"];
test_dc![clear_empty;"cf"];

test_dc![quit;"[qp]x10p"];
test_dc![no_quit_macro_depth;"[qp][x]x10p"];
test_dc![Quit;"[Qp]x10p"];
test_dc![no_Quit_macro_depth;"[Qp][x]x10p"];
test_dc![Quit_inconsistency;"1Q10p"];