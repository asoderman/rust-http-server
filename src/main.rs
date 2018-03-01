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

use cli::{run_cli, config_from_cli, cli_verbosity, cli_serve_directory};

pub static mut VERBOSE: bool = false;

fn main() {
    pretty_env_logger::init_custom_env("RUST_HTTP_SERVER_LOG");
    let cli = run_cli();
    cli_verbosity(&cli);

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
    
    let config = config_from_cli(&cli);

    let mut server = server::Server::from_config(config);

    cli_serve_directory(&cli, &mut server);

    server.serve();

    println!("Shutting down {}", name);
}

