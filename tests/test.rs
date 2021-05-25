/*
NOTE: These tests need to be run *manually*, e.g.
- plug nrf52840 DK in
- run `cargo test -- --ignored`
*/

use insta;

/// run probe-run with `args` and truncate the "Finished .. in .." and "Running `...`" flashing output
fn run_and_truncate(args: &str) -> String {
    let args = args.split(" ");

    let command = std::process::Command::new("cargo")
        .args(args)
        .output()
        .expect("failed to execute process");

    let probe_run_output = std::str::from_utf8(&command.stderr).unwrap();

    // remove the lines printed during flashing, as they contain timing info that's not always the same
    let mut truncated_probe_run_output = "".to_string();
    for line in probe_run_output.lines() {
        if !line.starts_with("    Finished") && !line.starts_with("     Running `") &&
            !line.starts_with("    Blocking waiting for file lock ") {
            truncated_probe_run_output.push_str(line);
            truncated_probe_run_output.push_str("\n");
        }
    }

    truncated_probe_run_output
}

#[test]
// this test should not be run by default, as it requires the target hardware to be present
#[ignore]
fn successful_run_has_no_backtrace() {
    let run_output = run_and_truncate("run -- --chip nRF52840_xxAA hello");

    insta::assert_snapshot!(run_output);
}

#[test]
// this test should not be run by default, as it requires the target hardware to be present
#[ignore]
fn successful_run_can_enforce_backtrace() {
    let run_output = run_and_truncate("run -- --chip nRF52840_xxAA hello --force-backtrace");

    insta::assert_snapshot!(run_output);
}
