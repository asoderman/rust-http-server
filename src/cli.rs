use clap::App;

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
            -v... 'Sets verbosity'")

}
