# embassy-pico-test

This repostitory contains code to test embassy-rs functions on the Raspberry Pi Pico.

## Using

You must have `Rust`, the `thumbv6m-none-eabi` and `thumbv8m.main-none-eabihf` targets, and `probe-rs` installed.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add thumbv6m-none-eabi thumbv8m.main-none-eabi
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
probe-rs complete install
```

You will need a Pico and a Debug Probe.  Connect both to your linux PC and connect the Debug Probe to the Pico.  See [this link](https://github.com/piersfinlayson/pico1541-rs/blob/main/BUILD.md#setting-up-a-pico-probe) for how to set up another Pico as a Debug Probe.

Build and run a test like this on the Pico:

```bash
cargo run --bin timing --features single-gpio,1
```

Where:
* `timing` is the type of tests to select from
* `single-gpio` is the sub-type of tests, from withing [`timing`](src/bin/timing.rs) to select from  
* `1` is the test number of to run, where the [source](src/bin/timing.rs) is the documentation

To run on the Pico 2:

```bash
cargo run --bin timing --no-default-features --features single-gpio,1,pico2
```

These commands will built the desired test, and then flash and restart the Pico or Pico 2 with the image.
