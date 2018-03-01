use std::usize;
use clap::App;

use config::{ConfigBuilder, Config};
use server::Server;

pub fn run_cli<'a, 'b>() -> App<'a, 'b> {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");
    App::new(name)
        .version(version)
        .author(author)
        .args_from_usage(
            "[DIRECTORY]            'Serves the contents of the directory'
            -t, --threads=[THREADS] 'Sets the number of threads to use'
            -h, --host=[HOST]       'Sets the host address'
            -p, --port=[PORT]       'Sets the port'
            -v...                   'Sets verbosity'")

}

pub fn config_from_cli(app: &App) -> Config {

    let matches = app.clone().get_matches();
    let mut config = ConfigBuilder::from_json_file("config.json").unwrap_or_default();


    if let Some(t) = matches.value_of("threads") {
        let threads = usize::from_str_radix(t, 10)
            .expect("Please enter an integer for thread size");
        config.set_threads(threads);
    }

    if let Some(host) = matches.value_of("host") {
        config.set_host(&host);
    }

    if let Some(port) = matches.value_of("port") {
        config.set_port(port);
    }

    config.build()
}

pub fn cli_verbosity(app: &App) {
    let matches = app.clone().get_matches();
    match matches.occurrences_of("v") {
        0 => set_verbosity(false),
        _ => set_verbosity(true),
    }
}

pub fn cli_serve_directory(app: &App, server: &mut Server) {
    let matches = app.clone().get_matches();
    if let Some(dir) = matches.value_of("DIRECTORY") {
        server.serve_directory(dir);
    }
}

fn set_verbosity(value: bool) {
    unsafe {
        ::VERBOSE = value;
    }
}
