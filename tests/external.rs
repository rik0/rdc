extern crate rdc;

use std::io;
use std::process;
use std::ffi::OsStr;

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

#[test]
fn test() {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let programs = vec!["10p"];
    let expected = run_dc(programs.clone()).expect("process error");
    let dc_args = prepare_arguments(programs);
    rdc::dc(dc_args.into_iter(), stderr, stdout);
    // assert_eq!(
    //     String::from_utf8(stdout).expect("utf8 output"),
    //     String::from_utf8(expected.stdout).expect("utf error in system dc output")
    // );
}
