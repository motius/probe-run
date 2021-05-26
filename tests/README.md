# Snapshot Tests go 📸✨

All tests in this directory are snapshot tests, e.g. they compare `probe-run` output to a previous, known-good state.

These tests need to be run *manually* because they require the target hardware to be present.

To do this,
1. connect a nrf52840 DK to your computer via the J2 USB port on the *short* side of the DK
2. run `cargo test -- --ignored`

## adding a new snapshot
refer to the [insta](https://docs.rs/insta/1.7.1/insta/#writing-tests) docs