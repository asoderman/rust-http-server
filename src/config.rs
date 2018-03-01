extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::env::args;
use std::io::prelude::*;
use std::io::Error;
use std::default::Default;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub app: Option<String>,
    pub static_folder: Option<String>,
    pub https_cert: Option<String>,
    pub cert_password: Option<String>,
    pub app_path: Option<String>,
    pub threads: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigBuilder {
    pub host: Option<String>,
    pub port: Option<String>,
    pub app: Option<String>,
    pub static_folder: Option<String>,
    pub https_cert: Option<String>,
    pub cert_password: Option<String>,
    pub app_path: Option<String>,
    pub threads: Option<usize>,
}

impl ConfigBuilder {

    pub fn from_json_file(file: &str) -> Result<Self, Error> {
        let mut f = File::open(file)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        // TODO: Fix the error handling here: convert serde_json error to io error
        let result: Self = serde_json::from_str(&contents).unwrap();

        Ok(result)
    }

    pub fn set_host(&mut self, host: &str) -> &mut Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn set_port(&mut self, port: &str) -> &mut Self {
        self.port = Some(port.to_string());
        self
    }

    pub fn set_app(&mut self, app: &str) -> &mut Self {
        self.app = Some(app.to_string());
        self
    }
    pub fn set_static_folder(&mut self, static_folder: &str) -> &mut Self {
        self.static_folder = Some(static_folder.to_string());
        self
    }

    pub fn set_https_cert(&mut self, https_cert: &str) -> &mut Self {
        self.https_cert = Some(https_cert.to_string());
        self
    }

    pub fn set_cert_password(&mut self, cert_password: &str) -> &mut Self {
        self.cert_password = Some(cert_password.to_string());
        self
    }

    pub fn set_threads(&mut self, threads: usize) -> &mut Self {
        self.threads = Some(threads);
        self
    }

    pub fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: self.port.unwrap_or_else(|| "8080".to_string()),
            app: self.app,
            static_folder: self.static_folder,
            https_cert: self.https_cert,
            cert_password: self.cert_password,
            app_path: None,
            threads: self.threads,
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            app: None,
            static_folder: None,
            https_cert: None,
            cert_password: None,
            app_path: None,
            threads: None,
        }
    }
}

impl Config {

    /// Tries to create a config from config.json falls back to 
    /// command line args
    pub fn new() -> Config {
        
        let file_result = from_file();

        match file_result {
            Ok(c) => c,
            Err(_) => from_args(),
        }
    }

    /// Create a config with provided information. This will be moved to a 
    /// builder pattern.
    // TODO: implement builder pattern for this type
    #[allow(dead_code)]
    pub fn from(host: Option<String>, port: Option<String>, app: Option<String>, static_folder: Option<String>, https_cert: Option<String>, cert_password: Option<String>, app_path: Option<String>) -> Config {

        let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = port.unwrap_or_else(|| "8080".to_string());

        Config {
            host,
            port,
            app,
            static_folder,
            https_cert,
            cert_password,
            app_path,
            threads: None,
        }
    }

    /// Create a config from a JSON string. Only used in testing currently.
    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Config {
        serde_json::from_str(json).unwrap()
    }

    /// Returns if HTTPS is enabled with this config.
    pub fn https(&self) -> bool {
        self.https_cert.is_some()
    }

}

fn from_args() -> Config {

    let arguments = args().collect();
    let host = get_host(&arguments).unwrap_or_else(|| "127.0.0.1".to_string());
    let port = get_port(&arguments).unwrap_or_else(|| "8080".to_string());
    Config {
        host,
        port,
        app: None,
        static_folder: None,
        https_cert: None,
        cert_password: None,
        app_path: None,
        threads: None,
    }
}

fn from_file() -> Result<Config, Error> {
    
    let mut f = File::open("config.json")?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    // TODO: Fix the error handling here: convert serde_json error to io error
    let result: Config = serde_json::from_str(&contents).unwrap();

    Ok(result)
}

/// Returns the host to run on. Taken from command line args.
#[cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
fn get_host(args: &Vec<String>) -> Option<String> {
    // TODO: port this to cli
    for argument in args {
        if argument.starts_with("host:") {
            return Some(argument.replace("host:", ""));
        }
    }
    None
}

/// Returns the port for the server to run on. Taken from command line args.
#[cfg_attr(feature = "cargo-clippy", allow(ptr_arg))]
fn get_port(args: &Vec<String>) -> Option<String> {
    // TODO: port this to cli
    for argument in args {
        let arg_string = argument;
        if arg_string.starts_with("port:") {
            return Some(arg_string.replace("port:", ""));
        }
    }
    None
}
