use insta;

/*
NOTE: These tests need to be run *manually*, e.g.
- plug nrf52840 DK in
- run `cargo test successful_run_has_no_backtrace`
*/

#[test]
// this test should not be run by default, as it requires the target hardware to be present
#[ignore]
fn successful_run_has_no_backtrace() {
    let command = std::process::Command::new("cargo")
        .args(&["run", "--", "--chip", "nRF52840_xxAA", "tests/test_elfs/hello"])
        .output()
        .expect("failed to execute process");

    let pobe_run_output = std::str::from_utf8(&command.stderr).unwrap();
    insta::assert_snapshot!(pobe_run_output);
}
