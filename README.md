# Rust-HTTP-Server
[![Build Status](https://travis-ci.org/asoderman/rust-http-server.svg?branch=master)](https://travis-ci.org/asoderman/rust-http-server)

Rust-HTTP-Server is a small multi-threaded webserver that supports TLS and WSGI. Rust-HTTP-Server's goal is to be an easy to use, fast and lightweight http server.

# Table of Contents:
* [Installation](#installation)
* [Usage](#usage)
* [TODO](#todo)

# Installation:
Currently the only way to install rust-http-server is to download the project and build with cargo. crates.io support is coming soon!

# Usage:
### With Cargo:
* `cargo run` to start the server
* `cargo run -- -h` for help
* `cargo test` to run the tests (Note: Openssl must be installed on your system otherwise the TLS test will fail.)

rust-http-server will look for `config.json` in the current working directory. Config options include:
* host
* port
* app - a string `<module>:<callable>`. The server will search for a file with the same name as the module to run.
* static_folder - A path to a folder. This will register **everything** within the folder.
* https_cert
* cert_password
* threads

## TODO:
* More documentation
* Fix setting host/port via CLI
