//! The server module.

extern crate native_tls;

use std::{str, env};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::sync::Arc;
use std::result::Result;
use std::fs::File;
use std::thread;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::fmt;

use self::native_tls::{TlsStream, TlsAcceptor, Pkcs12};

use config::Config;
use request::Request;
use response::Response;
use routing::Router;
use threadpool::ThreadPool;
//#[cfg(feature="wsgi")]
use wsgi::application::Application;

/// The server structure containing a config, a threadpool and a router.
pub struct Server {
    config: Config,
    thread_pool: ThreadPool,
    router: Router,
    dir: PathBuf,
}

#[derive(Debug)]
enum ServerError {
    ClientClosedConnection(&'static str),
}

type ServerResult<T> = Result<T, ServerError>;

trait Close{ 
    fn close(&mut self, how: Shutdown) -> ::std::io::Result<()>;
}

/// Close trait implementation for the `TlsStream` wrapper type.
/// The shutdown method has a different signature than the 
/// regular `TcpStream` so the Shutdown enum is handled in the close method.
/// The stream is only closed if the handler intends to shutdown write or both.
impl<T: Read + Write + Close> Close for TlsStream<T> {
    fn close(&mut self, how: Shutdown) -> ::std::io::Result<()> {
        match how {
            Shutdown::Both | Shutdown:: Write => {
                self.shutdown()
            }
            Shutdown::Read => { Ok(()) }
        }
    }
}
impl Close for TcpStream {
    fn close(&mut self, how: Shutdown) -> ::std::io::Result<()> {
        self.shutdown(how)
    }
}

trait Connection: Read + Write + Close {}
impl<T> Connection for T where T: Read + Write + Close {}

pub trait ServerApplication {
    fn create(app_string: Option<&String>, port: &str) -> Option<Self> where Self: ::std::marker::Sized;
    fn handle_one_request(&self, Request) -> Result<String, InternalServerError>;
}

/// Error type for applications
#[derive(Debug)]
pub struct InternalServerError(pub String, pub Option<&'static Error>);

impl Error for InternalServerError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InternalServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", self.0)
    }
}

impl Server {
    // TODO: Implement a server shutdown method that stops tcp listeners
    // this can be done with mpsc I believe.

    /// Creates a server with provided config object.
    pub fn from_config(config: Config) -> Server {
        let threads = config.threads.unwrap_or(1);
        let thread_pool = ThreadPool::new(threads);

        let static_folder = config.static_folder.clone();

        let router = Router::from(&static_folder
                                  .unwrap_or_else(|| "static".to_string()));

        // This unwrap should probably changed to a default directory
        let dir = env::current_dir().unwrap();

        Server {
            config,
            thread_pool,
            router,
            dir,
        }
    }

    pub fn serve_directory(&mut self, dir: &str) {
        trace!("Registering static routes for: {}", dir);
        self.router.register_static_routes(dir);
    }

    /// The main loop of the program.
    /// Listens on specified host and port and accepts incoming connections
    /// If a pkcs12 is provided the server will listen on port 8443 for HTTPS
    /// requests as well. The pkcs12 password can be provided in the config or 
    /// as an environment variable.
    /// Takes requests and adds them and their handler to the threadpool.
    pub fn serve(&self) {
        let listener = TcpListener::bind(format!("{}:{}", &self.config.host, &self.config.port)).expect("Could not start listener on specified host address/port");
        info!("Running on host: {}", &self.config.host);
        info!("Running on port: {}", &self.config.port);

        let app = Arc::new(Application::create(self.config.app.as_ref(), &self.config.port)); // This will probably be changed
        let shared_router = Arc::new(self.router.clone());
        
        let https: bool = self.config.https();
        
        if https {
            
            info!("HTTPS enabled. Running on {}:8443", &self.config.host);
            let second_listener = TcpListener::bind(format!("{}:8443", &self.config.host)).expect("Unable to create TCP listener on specified HTTPS port.");
            let https_app = app.clone();
            let https_router = shared_router.clone();

            // TLS 
            let cert_filename = self.config.https_cert.clone().unwrap();
            let password =  env::var("PKCS12_PASSOWRD")
                .unwrap_or_else(|_| self.config.cert_password
                           .clone()
                           .expect("Please provide a password for the pkcs12 \
                                   bundle, either as an environment variable or \
                                   in the config as 'cert_password'"));
            vprintln!("cert path {:?}", Path::join(&self.dir, &cert_filename));
            let mut file = File::open(Path::join(&self.dir, &cert_filename))
                       .expect("Could not open certificate.");
            let mut pkcs12= vec![];
            file.read_to_end(&mut pkcs12).expect("Could not read identity.pfx");
            let pkcs12 = Pkcs12::from_der(&pkcs12, &password)
                .expect("Could not open pkcs12");
            let acceptor = TlsAcceptor::builder(pkcs12)
                .unwrap()
                .build()
                .expect("Could not build TlsAcceptor. Please verify the file \
                        provided is a pkcs12.");
            let acceptor = Arc::new(acceptor);

            let executor = self.thread_pool.create_executor();

            thread::spawn(move || {
                for stream in second_listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            let app_instance = https_app.clone();
                            let router_instance = https_router.clone();
                            let acceptor = acceptor.clone();
                            let mut stream = match acceptor.accept(stream) {
                                Ok(stream) => stream,
                                Err(e) => {
                                    error!("Error accepting TLS connection
                                    {:?}", e);
                                    continue;
                                }
                            };
                            executor(move || { 
                                if let Err(e) = 
                                    handle_connection(&mut stream,
                                                      app_instance.as_ref(),
                                                      &router_instance) {
                                error!("Error handling connection {:?}", e);
                                };
                            });
                        }
                        Err(e) => { error!("There was an error opening connection
                                           {:?}", e); }
                    }
                }
            });
        }

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let app_instance = app.clone();
                    let router_instance = shared_router.clone();
                    self.thread_pool.execute(move || {
                        if let Err(e) = handle_connection(&mut stream, app_instance.as_ref(), &router_instance) {
                            error!("Error handling connection {:?}", e);
                        };
                    });
                }
                Err(e) => { error!("There was an error opening the connection. 
                                   {:?}", e); }
            }
        }
    }
}


/// Handles each connection in its own thread.
/// Passes the request to the application then once the application
/// has created a response sends the response.
// TODO: Make the app parameter generic for an Application trait (e.g. wsgi or 
// something else)
fn handle_connection<T: Connection>(stream: &mut T,
                                    app: &Option<Application>,
                                    router: &Router) -> ServerResult<()>{

    let mut buf = [0u8; 256]; // buffer size = 256 bytes
    match stream.read(&mut buf) {
        Ok(_) => { 
            let data = str::from_utf8(&buf).expect("Could not convert request data to utf8");
            let request = Request::from(String::from(data));
            vprintln!("REQUEST: {}", request);
            info!("Handling request: {}", request);
            match stream.close(Shutdown::Read) { 
                Ok(_) => {
                    if router.is_static_content(&request.path) {
                        let result = match serve_static_content(&request, router) {
                            Ok(data) => {
                            Response::http_ok_file(data).to_bytes()
                        },
                            Err(e) =>  {
                                error!("Error loading static content {:?}", e);
                                Response::server_error().to_bytes()
                            }
                        };
                        if let Err(e) = stream.write(&result) {
                            error!("Error writing to stream: {:?}", e);
                        };
                    } else {
                        let result = app.clone()
                            .map_or(Response::not_found().to_string(), |app| { 
                                match app.handle_one_request(request){
                                    Ok(v) => v,
                                    Err(e) => {
                                        error!("Application error: {:?}", e);
                                        Response::server_error().to_string()
                                    }
                                }
                            });
                        if let Err(e) = stream.write(result.as_bytes()) {
                            error!("Error writing to stream: {:?}", e);
                        };
                    }
                    if let Err(e) = stream.close(Shutdown::Write) {
                        error!("Error closing stream: {:?}", e);
                    } 
                    vprintln!("Stream has been flushed");
                    Ok(())
                }
                Err(_) => {
                    Err(ServerError::ClientClosedConnection("Error closing read connection"))
                }
            }
        }
        Err(_) =>  Err(ServerError::ClientClosedConnection("Connection closed by client"))
    }
}

/// Gets the absolute path of the file and reads it into a buffer. Returns 
/// the buffer.
fn serve_static_content(request: &Request, router: &Router) -> Result<Vec<u8>, ::std::io::Error> {
    let abs_path = router.get(&request.path);
    let mut buffer = Vec::new();
    let mut file = File::open(&abs_path)?;
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    /// Bug: tests can fail due to not being able to listen on a port
    extern crate reqwest;
    extern crate curl;
    extern crate tempdir;

    use super::*;
    use std::process::Command;
    use std::thread;
    use self::curl::easy::Easy;
    use std::path::Path;
    use std::fs::{File, create_dir};

    /// Run a server in a seperate thread then make a request to the server.
    /// Assert the response is a success
    #[test]
    fn test_serve() {
        
        // Create resources for the test
        let (static_path, _test_dir) = create_test_dir().unwrap();

        let test_port = "9999";
        let config = create_test_config(test_port);
        let mut server = Server::from_config(config);
        server.router.register_static_routes(static_path.as_str());
        thread::spawn(move || { server.serve(); } );

        let response = reqwest::get(&format!("http://127.0.0.1:{}/static/index.html", test_port)).unwrap();
        assert!(response.status().is_success());
    }

    #[test]
    fn test_serve_can_404() {

        let test_port = "9998";
        let config = create_test_config(test_port);
        let server = Server::from_config(config);

        thread::spawn(move || { server.serve(); } );

        let response = reqwest::get(&format!("http://127.0.0.1:{}/index", test_port)).unwrap();
        assert!(response.status().is_client_error());

    }

    #[test]
    fn test_serve_can_500() {
        let test_port = "5000";
        let config = create_test_config_fail_app(test_port);
        let server = Server::from_config(config);

        thread::spawn(move || { server.serve(); } );

        let response = reqwest::get(&format!("http://127.0.0.1:{}/", test_port)).unwrap();
        assert!(response.status().is_server_error());

    }
    fn create_test_config_fail_app(port: &str) -> Config {
        let text = r#"{
        "host": "127.0.0.1",
        "port": "{port}",
        "app" : "failure:app"
        }"#.to_string();
        let result = text.replace("{port}", port);

        Config::from_json(&result)
    }

    /// A test configuration in JSON
    fn create_test_config(port: &str) -> Config {
        let text = r#"{
        "host": "127.0.0.1",
        "port": "{port}"
        }"#.to_string();
        let result = text.replace("{port}", port);

        Config::from_json(&result)
    }
    fn create_test_dir() -> ::std::io::Result<(String, tempdir::TempDir)> {
        let current = env::current_dir()?;
        let test_dir = tempdir::TempDir::new_in(&current, "server-test")?;
        let mut static_path = test_dir.path().join("static");
        create_dir(&static_path)?;
        static_path.push("index.html");
        let f = File::create(&static_path)?;
        f.sync_all().expect("Could not sync test file to filesystem");
        static_path.set_file_name("");

        return Ok((String::from(static_path.to_str().unwrap()), test_dir))

    }

    /// Uses rust curl bindings because reqwest does not provide an easy way to
    /// allow self signed certs
    #[test]
    fn test_https_serve() {
        create_test_cert().expect("Test self signed certificate could not be created");
        let mut buf = Vec::new();
        File::open("cert.pem").unwrap().read_to_end(&mut buf).unwrap();

        let (static_path, _test_dir) = create_test_dir().unwrap();

        let config = create_test_https_config();
        let mut server = Server::from_config(config);
        server.router.register_static_routes(static_path.as_str());

        thread::spawn(move || { server.serve(); } );

        let mut response = Easy::new();
        response.ssl_verify_host(false)
            .expect("Could not disable SSL host verification");
        response.ssl_verify_peer(false)
            .expect("Could not disable SSL peer verification");
        response.url("https://127.0.0.1:8443/static/index.html").unwrap();
        response.perform().unwrap();


        assert!(response.response_code().unwrap() > 199 && 
                response.response_code().unwrap() < 400);
    }

    /// A test configuration in JSON
    fn create_test_https_config() -> Config {
        let text = r#"{
        "host": "127.0.0.1",
        "port": "8080",
        "https_cert": "test.pfx",
        "cert_password": "password"
        }"#;

        Config::from_json(text)
    }

    /// Automates creating a self signed certificate.
    fn create_test_cert() -> ::std::io::Result<()> {
        if !Path::new("test.pfx").exists() {
            let openssl = "openssl";
            let create_cert_args = ["req", "-x509", 
            "-newkey", "rsa:2048", 
            "-keyout", "key.pem", 
            "-out", "cert.pem", 
            "-days", "365", 
            "-nodes", 
            "-subj", "/CN=localhost"];
            let create_pkcs12 = ["pkcs12", "-export", 
            "-out","test.pfx", 
            "-inkey", "key.pem", 
            "-in", "cert.pem", 
            "-passout", "pass:password"];

            let cwd = ::std::env::current_dir().unwrap();
            let _ = Command::new(&openssl)
                            .current_dir(&cwd)
                            .args(&create_cert_args)
                            .output()?;
            let _ = Command::new(&openssl)
                         .current_dir(&cwd)
                         .args(&create_pkcs12)
                         .output()?;
        }

        Ok(())

    }
}
