extern crate cpython;

use std::env;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::error::Error;
use std::fmt;
use std::convert::From;

use self::cpython::{Python, PythonObject, PyString, PyList, PyDict, PythonObjectWithCheckedDowncast, PyResult, PyErr};
use utils::file::locate_file;
use request::Request;
use server::{ServerApplication, InternalServerError};

type PythonResult<T> = Result<T, PythonError>;

/// Wrapper type for `PyErr` because `PyErr` does not implement Error trait
#[derive(Debug)]
pub struct PythonError(PyErr);

impl Error for PythonError {
    fn description(&self) -> &str {
        "Error calling python application."
    }
}

impl fmt::Display for PythonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} ({:?})", self.0.ptype, self.0.pvalue)
    }
}

impl From<PythonError> for InternalServerError {
    fn from(p: PythonError) -> Self {
        Self{0: p.description().to_string(), 1: None}
    }
}

/// Wraps `PyErr` in this crates `PythonError` type. This is to allow conversion to 
/// other error types.
fn convert_error<T>(p: PyResult<T>) -> PythonResult<T> {
    match p {
        Ok(v) => Ok(v),
        Err(e) => Err(PythonError(e))
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    module: String,
    callable: String,
    headers_set: Vec<String>,
    path_to_app: PathBuf,
    path_to_bindings: String,
    port: String,
}

impl Application { 

    /// Takes a string "<module>:<app>" returns tuple (module, app)
    fn parse_app_string(string: &str) -> (String, String) {
        let split: Vec<&str> = string.split(':').collect();
        (split[0].to_string(), split[1].to_string())
    }

    /// Calls the wsgi application to generate a HTTP response.
    fn call_application(&self, request: Request, py: Python) -> PyResult<String> {
        let locals = PyDict::new(py);

        let env = self.set_env(request, py)?;

        let sys = py.import("sys")?;
        let sys_path = sys.get(py, "path")?;
        let path_as_py_object = PyString::new(py, self.path_to_app.to_str().unwrap_or("")).into_object();
        let bindings_path_as_py_object = PyString::new(py, &self.path_to_bindings).into_object();
        let path_list = PyList::downcast_from(py, sys_path)?;
        path_list.insert_item(py, 0, path_as_py_object);
        path_list.insert_item(py, 1, bindings_path_as_py_object);

        let imported_module = py.import(&self.module)?;
        locals.set_item(py, &self.module.clone(), imported_module)?;
        locals.set_item(py, "bind", py.import("bindings")?)?;
        locals.set_item(py, "env", env)?;

        let call_statement = format!("bind.Application.call_callable(env, {}.{})", self.module, self.callable);

        let result = py.eval(&call_statement, None, Some(&locals))?;
        let value = result.extract(py)?;
        Ok(value)
    }

    /// Generates the python wsgi bindings.
    fn create_bindings() -> String {
        // TODO: Error handling
        let mut path = env::current_exe().unwrap();
        path.set_file_name("bindings.py");
        if Path::exists(&path) { 
            String::from(path .to_str().unwrap())
        } else {
            static BINDINGS_TEXT: &[u8] = include_bytes!("bindings.py");
            let mut file = File::create(&path).unwrap();
            file.write_all(BINDINGS_TEXT).expect("Could not write WSGI bindings");
            String::from(path.to_str().unwrap())
        }
    }

    /// Sets the wsgi env to prepare the application to handle a request.
    fn set_env(&self, request: Request, py: Python) -> PyResult<PyDict> {
        let env = PyDict::new(py);
        env.set_item(py, "wsgi.version", "1.0")?;
        env.set_item(py, "wsgi.url_scheme", "http")?;
        env.set_item(py, "wsgi.input", request.data)?;
        env.set_item(py, "wsgi.errors", "2>")?;
        env.set_item(py, "wsgi.multithread", true)?;
        env.set_item(py, "wsgi.multiprocess", true)?;
        env.set_item(py, "wsgi.run_once", false)?;
        
        env.set_item(py, "REQUEST_METHOD", format!("{}", request.kind))?;
        env.set_item(py, "PATH_INFO", request.path)?;
        env.set_item(py, "SERVER_NAME", format!("{}", request.host))?; 
        env.set_item(py, "SERVER_PORT", format!("{}", self.port))?;
        Ok(env)
    }
}

impl ServerApplication for Application {

    /// The constructor for a wsgi Application.
    fn create(app_string: Option<&String>, port: &str) -> Option<Application> {
        let app_string = app_string?;

        let port = port.to_string();

        let (module, callable) = Application::parse_app_string(app_string);
        let headers_set = Vec::new();
        let path_to_app = match locate_file(&module) {
            Some(location) => location,
            None => return None,
        };
        let created_at = Application::create_bindings();
        let mut created_at = PathBuf::from(created_at);
        created_at.set_file_name("");
        let path_to_bindings = String::from(created_at.to_str().unwrap());

        let app = Application {
            module,
            callable,
            headers_set,
            path_to_app,
            path_to_bindings,
            port,
        };
        Some(app)
    }
    
    fn handle_one_request(&self, request: Request)-> Result<String, InternalServerError> {
        let gil = Python::acquire_gil();
        let result = convert_error(self.call_application(request, gil.python()))?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    extern crate reqwest;

    use std::env;
    use std::thread;
    use config::Config;
    use server::Server;

    #[test]
    fn test_flask_integration() {
        let mut current = env::current_dir().unwrap();
        current.push("src/wsgi/test_scripts");
        let test_port = "9595";
        let config = create_test_config(&test_port, &current.to_str().unwrap());
        let server = Server::from_config(config);

        thread::spawn(move || { server.serve() } );

        // NOTE: use threadpool instead of thread for cleanup
        let mut request = reqwest::get(&format!("http://127.0.0.1:{}", test_port)).unwrap();

        assert!(request.status().is_success());
        assert_eq!(request.text().unwrap(), "Hello, World!");

    }

    fn create_test_config(port: &str, location: &str) -> Config {
        let text = r#"{
        "host": "127.0.0.1",
        "port": "{port}",
        "app": "flask_test_app:app",
        "app_path": "{location}"
        }"#.to_string();
        let result = text.replace("{port}", port);
        let result = result.replace("{location}", location);

        Config::from_json(&result)
    }
}
