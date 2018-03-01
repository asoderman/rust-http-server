//! The servers router module.
extern crate walkdir;

use std::collections::HashMap;
use std::path::{Path, Component};
use std::ffi::OsStr;

use self::walkdir::WalkDir;

use utils::file::is_hidden;

/// Stores routes in a hashmap. Checks if the request is trying to access a static
/// resouce and provides file location for the resource. Only files in the specified
/// folder(s) are visible on the server. Static routes are generated on server
/// startup therefore inorder for changes to take effect the server must be 
/// restarted.
#[derive(Clone)]
pub struct Router {
    pub static_routes: HashMap<String, String>,
}

impl Router {
    /// Initialize a `Router` without any routes.
    pub fn new() -> Router {
        let static_routes = HashMap::new();

        Router {
            static_routes
        }
    }

    /// Initialize a `Router` and immediately register routes for the directory provided.
    pub fn from(folder: &str) -> Router {
        let mut router = Router::new();
        router.register_static_routes(folder);
        router
    }

    /// Creates routes for files within a directory.
    pub fn register_static_routes(&mut self, folder: &str) {
        vprintln!("Registering routes for: {}", &folder);
        let directory = Path::new(&folder);
        let folder = directory.file_name().map_or("", |f| f.to_str().unwrap());

        for entry in WalkDir::new(&directory).into_iter()
            .filter_entry(|e| !is_hidden(e) ) {
            let entry = match entry {
                Ok(e) => e,
                _ => continue,
            };
            let is_file = entry.metadata().ok().map_or(false, |m| m.is_file());
            if is_file {
                let url = Router::create_url(entry.path(), folder);
                let abs_path = entry.path();
                self.static_routes.insert(url, 
                                          abs_path.to_str().unwrap().to_string());
            }
        }
        vprintln!("Routes: {:?}", self.static_routes);
    }

    /// Creates a URL for the resource.
    /// Example:
    /// A user is working in directory `app`.
    /// The user registers the directory `static` within `app`
    /// The URLs for the files with in static will be `/static/foo.js` etc.
    fn create_url(path: &Path, dir: &str) -> String {
        let components = path.components();

        let filtered: Vec<&str> = components
            .skip_while(|c| Some(*c) != Some(Component::Normal(OsStr::new(dir))))
            .map(|c| c.as_os_str())
            .filter_map(|c| c.to_str())
            .collect();
        let result = filtered.join("/");
        if result.starts_with('/') {
            result
        } else {
            format!("/{}", result)
        }
    }

    /// Checks if the route provided is an actual resource.
    pub fn is_static_content(&self, path: &str) -> bool {
        self.static_routes.contains_key(path)
    }

    /// Retrieves the full path to the resource.
    pub fn get(&self, path: &str) -> &str {
        &self.static_routes[path]
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use super::*;
    use std::fs::File;
    use self::tempdir::TempDir;

    #[test]
    fn test_register_static_routes() {
        // Check if the size of the hash table is 1 after adding a file
        let test_dir = TempDir::new("rhs-tests").unwrap();
        let filepath = test_dir.path().join("test.txt");
        let _ = File::create(filepath).unwrap();

        let mut router = Router::new();
        assert_eq!(0, router.static_routes.len());

        router.register_static_routes(test_dir.path().to_str().unwrap());

        assert_eq!(1, router.static_routes.len());
    }

    #[test]
    fn test_route_checking() {

        let test_dir = TempDir::new("rhs-tests").unwrap();
        let path_str = test_dir.path().file_name().unwrap().to_str().unwrap();
        let filepath = test_dir.path().join("test.txt");
        let _ = File::create(filepath).unwrap();

        let router = Router::from(test_dir.path().to_str().unwrap());
        println!("path prefix in route: {}", path_str);
        println!("routes: {:?}", router.static_routes);
        assert!(router.is_static_content(&format!("/{}/test.txt", path_str)));
        assert!(!router.is_static_content(&format!("/{}/this_does_not_exist.txt", path_str)));
    }

    #[test]
    fn test_preserve_directory_structure() {
        let test_dir = TempDir::new("rhs-tests")
            .expect("Could not create test dir");
        let test_dir_name = test_dir.path().file_name().unwrap().to_str().unwrap();
        let path = test_dir.path();
        let sub_dir = TempDir::new_in(&test_dir, "foo")
            .expect("Could not create sub dir");
        let sub_dir_name = sub_dir.path().file_name().unwrap().to_str().unwrap();
        let file_name = sub_dir.path().join("test.txt");
        println!("{:?}", file_name);
        let _ = File::create(file_name.as_path()).unwrap();

        let router = Router::from(path.to_str().unwrap());
        println!("routes: {:?}", router.static_routes);

        assert!(
            router
            .is_static_content(
                &format!("/{test_dir}/{sub_dir}/test.txt", 
                         test_dir=test_dir_name,
                         sub_dir=sub_dir_name)));

    }

}
