use insta;
use os_pipe::pipe;
use std::io::Read;
use std::sync::Mutex;
use structopt::lazy_static::lazy_static;

lazy_static! {
    /// rust will try to run the tests in parallel by default, and `insta` doesn't like the
    /// usual way of disabling this via `--test-threads=1`, so we're using this
    /// mutex to make sure we're not re-flashing until the last run is finished
    static ref ONE_RUN_AT_A_TIME: Mutex<i32> = Mutex::new(0i32);
}

/// run probe-run with `args` and truncate the "Finished .. in .." and "Running `...`" flashing output
/// NOTE: this currently only capures `stdin`, so any `log::` ed output, like flashing
fn run_and_truncate(args: &str) -> String {
    let _guard = ONE_RUN_AT_A_TIME.lock().unwrap();

    let args = args.split(" ");
    let (mut reader, writer) = pipe().unwrap();
    let writer_clone = writer.try_clone().unwrap();

    let mut command = std::process::Command::new("cargo");
    command.args(args);

    // capture stderr and stdout while preserving line order
    command.stdout(writer);
    command.stderr(writer_clone);

    // run `probe-run`
    let mut handle = command.spawn().unwrap();

    // Very important when using pipes: This parent process is still
    // holding its copies of the write ends, and we have to close them
    // before we read, otherwise the read end will never report EOF. The
    // Command object owns the writers now, and dropping it closes them.
    drop(command);

    // retrieve output and clean up
    let mut probe_run_output = String::new();
    reader.read_to_string(&mut probe_run_output).unwrap();
    handle.wait().unwrap();

    // remove the lines printed during flashing, as they contain timing info that's not always the same
    let mut truncated_probe_run_output = "".to_string();
    for line in probe_run_output.lines() {
        if !line.starts_with("    Finished")
            && !line.starts_with("     Running `")
            && !line.starts_with("    Blocking waiting for file lock ")
            && !line.starts_with("   Compiling probe-run v")
        {
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
    let run_output = run_and_truncate("run -- --chip nRF52840_xxAA tests/test_elfs/hello");

    insta::assert_snapshot!(run_output);
}

#[test]
// this test should not be run by default, as it requires the target hardware to be present
#[ignore]
fn successful_run_can_enforce_backtrace() {
    let run_output =
        run_and_truncate("run -- --chip nRF52840_xxAA tests/test_elfs/hello --force-backtrace");
    insta::assert_snapshot!(run_output);
}
