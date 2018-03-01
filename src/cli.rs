use std::usize;
use clap::{App, ArgMatches};

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
            -h, --host=[HOST]       'Sets the host address'
            -p, --port=[PORT]       'Sets the port'
            -a, --app=[APP]         '<module>:<callable> The server application'
            --cert=[CERT]           'Path to pkcs12 certificate'
            --pwd=[PWD]             'Password for the pkcs12'
            -t, --threads=[THREADS] 'Sets the number of threads to use'
            -v...                   'Sets verbosity'")

}

pub fn config_from_cli(args: &ArgMatches) -> Config {
    let mut config = ConfigBuilder::from_json_file("config.json").unwrap_or_default();

    if let Some(t) = args.value_of("threads") {
        let threads = usize::from_str_radix(t, 10)
            .expect("Please enter an integer for thread size");
        config.set_threads(threads);
    }

    if let Some(host) = args.value_of("host") {
        config.set_host(host);
    }

    if let Some(port) = args.value_of("port") {
        config.set_port(port);
    }

    if let Some(app) = args.value_of("app") {
        config.set_app(app);
    }

    if let Some(cert) = args.value_of("cert") {
        config.set_https_cert(cert);
    }

    if let Some(pwd) = args.value_of("pwd") {
        config.set_cert_password(pwd);
    }

    config.build()
}

pub fn cli_verbosity(app: &ArgMatches) {
    match app.occurrences_of("v") {
        0 => set_verbosity(false),
        _ => set_verbosity(true),
    }
}

pub fn cli_serve_directory(app: &ArgMatches, server: &mut Server) {
    if let Some(dir) = app.value_of("DIRECTORY") {
        server.serve_directory(dir);
    }
}

fn set_verbosity(value: bool) {
    unsafe {
        ::VERBOSE = value;
    }
}
