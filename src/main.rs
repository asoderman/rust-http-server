//! #rust-http-server
//!
//! An HTTP server written in rust
//! 
#[macro_use] extern crate serde_derive;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate ctrlc;
extern crate clap;

#[macro_use]
mod utils;

mod server;
mod threadpool;
mod config;
mod response;
mod request;
mod routing;
//#[cfg(feature="wsgi")]
mod wsgi;
mod cli;

use std::process;
use std::usize;

use cli::run_cli;

pub static mut VERBOSE: bool = false;

fn main() {
    pretty_env_logger::init_custom_env("RUST_HTTP_SERVER_LOG");
    let cli = run_cli();

    let name = env!("CARGO_PKG_NAME");
    println!("Running {}", name);
    if !log_enabled!(log::Level::Trace) {
        eprintln!("Logger is not enabled. To enable set env var `RUST_HTTP_SERVER_LOG=log=trace`")
    }
    // Listen for keyboard interrupt here
    ctrlc::set_handler(move || {
        // TODO: Implement cleanup instead of just exiting
        process::exit(0);
    }).expect("Error setting Ctrl-c handler");
    
    let mut server = server::Server::new();

    process_cli(cli, &mut server);

    server.serve();

    println!("Shutting down {}", name);
}

fn process_cli(app: clap::App, server: &mut server::Server) {

    let matches = app.get_matches();

    match matches.occurrences_of("v") {
        0 => set_verbosity(false),
        _ => set_verbosity(true),
    }

    if let Some(t) = matches.value_of("threads") {
        let threads = usize::from_str_radix(t, 10)
            .expect("Please enter an integer for thread size");
        server.set_threads(threads);
    }

    if let Some(dir) = matches.value_of("DIRECTORY") {
        server.serve_directory(dir);
    }
}

fn set_verbosity(value: bool) {
    unsafe {
        VERBOSE = value;
    }
}
