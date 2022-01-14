# Caduceus, a CLI client for Hermes

This repository holds the source code for Caduceus, a command line interface for Hermes (currently in the other subfolder of this repository).
It allows for the addition of reminders to Hermes.

To compile Caduceus, simply run `cargo build`. To run Caduceus, `cargo run -- [OPTION]` will run Caduceus with the provided option as a command line argument.
The options `-h` or `help` will list available options.

For long term use, an alternative to cargo run is to build a symbolic link to the executable produced by cargo, allowing it to be used as `caduceus [OPTION]`, as would be more natural. This executable can be found under `./target/debug/` when compiled in debug mode (Cargo doesn't allow convenient configuration of the output location, unfortunately). A quick search on symbolic links in Linux should suitably explain the process.

To build in release mode, use `cargo build --release`. The executable will be under `./target/release/`.

## Dependencies

[ZeroMQ](https://zeromq.org/) is the only dependency of Caduceus. It is also a dependency of Hermes itself, so if you have Hermes built and running this dependency should be satisfied.

## Name

Caduceus is named after the staff carried by Hermes in mythology, a symbol of messengers.