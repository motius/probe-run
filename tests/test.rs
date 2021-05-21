use insta;
use std::process::Stdio;
use std::io::Read;

#[test]
fn successful_run_has_no_backtrace() {
    /*
    let test_opts = Opts {
        list_chips: false,
        list_probes: false,
        chip: Some("nRF52840_xxAA"),
        probe: None,
        speed: None,
        elf: Some("test_elfs/hello"),
        no_flash: false,
        connect_under_reset: false,
        verbose: 0,
        version: false,
        force_backtrace: false,
        max_backtrace_len: 50,
        shorten_paths: false,
        _rest: [],
    };


    probe_run.notmain(test_opts);
    */

    let command = std::process::Command::new("cargo")
        .args(&["run", ".."])
        .output()
        .expect("failed to execute process");

    let stdout_buffer = std::str::from_utf8(&command.stderr);

    println!("xoxo {:?}", stdout_buffer);

}
