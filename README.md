# Rust-HTTP-Server
[![Build Status](https://travis-ci.org/asoderman/rust-http-server.svg?branch=master)](https://travis-ci.org/asoderman/rust-http-server)
[![Crates.io](https://img.shields.io/crates/v/rust-http-server.svg)](https://crates.io/crates/rust-http-server)

Rust-HTTP-Server is a small multi-threaded webserver that supports TLS and WSGI. Rust-HTTP-Server's goal is to be an easy to use, fast and lightweight http server.

# Table of Contents:
* [Installation](#installation)
* [Usage](#usage)
* [TODO](#todo)

# Installation:

### macOS:

#### Homebrew
Tap this repo then install via homebrew.

`brew tap asoderman/rust-http-server https://github.com/asoderman/rust-http-server`
`brew install asoderman/rust-http-server/rust-http-server`

#### Cargo:
`cargo install rust-http-server` will work as long as you have a homebrew install of python. If you 
are still using the python install provided by Apple run `brew reinstall python` first.

### Ubuntu:

#### Cargo:
This will install system dependencies and then install the server via cargo.

`sudo apt-get install -y pkg-config libssl-dev python3-dev && cargo install rust-http-server`

# Usage:
* `rust-http-server` to start the server
* `rust-http-server --help` for help
* `cargo test` to run the tests (Note: Openssl must be installed on your system otherwise the TLS test will fail.)

rust-http-server will look for `config.json` in the current working directory. Config options include:
* host
* port
* app - a string `<module>:<callable>`. The server will search for a file with the same name as the module to run.
* static_folder - A path to a folder. This will register **everything** within the folder.
* https_cert
* cert_password
* threads

CLI usage:
```

            [DIRECTORY]            'Serves the contents of the directory'
            -h, --host=[HOST]       'Sets the host address'
            -p, --port=[PORT]       'Sets the port'
            -a, --app=[APP]         '<module>:<callable> The server application'
            -cert=[CERT],           'Path to pkcs12 certificate'
            -pwd=[PWD],             'Password for the pkcs12'
            -t, --threads=[THREADS] 'Sets the number of threads to use'
            -v...                   'Sets verbosity'
```

## TODO:
* More documentation
* HTTP passthrough
* Benchmarks
