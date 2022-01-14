# Hermes, A Reminder Daemon for Linux

This sub-directory holds the source code for Hermes, a Linux [Daemon](https://en.wikipedia.org/wiki/Daemon_(computing)) for convenient desktop notification-based reminders, written in [Rust](https://www.rust-lang.org/). The other sub-directory in the repository contains Caduceus, a command line interface for interacting with Hermes, to set up notifications.

To compile Hermes, simply run `cargo build`, or `cargo build --release`. The binary will be placed in `./target/debug/`, or `./target/release/`, as Cargo does not currently have options for changing the location of the final executable. Running Hermes, either with `cargo run`, or just running the executable itself, starts the daemon.

## Dependencies

Hermes has a small dependency set, listed below. Underlying dependencies of ZeroMQ, DBus, or SQLite can be installed via your Linux distribution's package manager.

__libc__ - Hermes relies on the libc crate to interface with libc for the daemonization process, as Rust provides no API for the fork() and related functions needed.

[__ZeroMQ__](https://zeromq.org/) - ZeroMQ is a library for simple internal sockets, used in Hermes to listen to messages from other processes, such as a client like Caduceus.

__DBus__ - In order to send notifications to the desktop, DBus is used as the message bus interface. Most desktop environments in Linux have this installed.

__SQLite__ - In order to store data longterm, Hermes uses SQLite, a lightweight file database, to store reminder information.

