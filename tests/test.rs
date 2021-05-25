/*
NOTE: These tests need to be run *manually*, e.g.
- plug nrf52840 DK in
- run `cargo test -- --ignored`
*/

#[cfg(test)]

mod target_tests {
    use insta;

    #[test]
    // this test should not be run by default, as it requires the target hardware to be present
    #[ignore]
    fn successful_run_has_no_backtrace() {
        let command = std::process::Command::new("cargo")
            .args(&["run", "--", "--chip", "nRF52840_xxAA", "tests/test_elfs/hello"])
            .output()
            .expect("failed to execute process");

        let probe_run_output = std::str::from_utf8(&command.stderr).unwrap();

        // remove the lines printed during flashing, as they contain timing info that's not always the same
        let mut truncated_probe_run_output = "".to_string();
        for line in probe_run_output.lines() {
            if !line.starts_with("    Finished") &&
               !line.starts_with("     Running `") {
                    truncated_probe_run_output.push_str(line);
                    truncated_probe_run_output.push_str("\n");
                }
            }

        insta::assert_snapshot!(truncated_probe_run_output);
    }

}