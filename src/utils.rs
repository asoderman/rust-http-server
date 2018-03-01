//! A module for misc. functions.

pub mod file {
    extern crate walkdir;

    use self::walkdir::{WalkDir, DirEntry};
    use std::env;
    use std::path::PathBuf;

    /// Takes a filestem and searches for the file in the current directory.
    /// returns the directory that the file is located in.
    pub fn locate_file(module: &str) -> Option<PathBuf> {
        let cwd = env::current_dir().unwrap();
        
        for entry in WalkDir::new(cwd.as_path())
            .into_iter()
            .filter_entry(|e| !is_hidden(e)) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let is_file = entry.metadata()
                    .ok()
                    .map_or(false, |m| m.is_file());
                if is_file {
                    let stem = entry.path().file_stem();
                    let stems_match = stem.map_or(false, |s| s == module);
                    if stems_match {
                        let mut buf = entry.path().to_path_buf();
                        buf.set_file_name("");
                        return Some(buf)
                    }
                }
        }
        None
    }

    pub fn is_hidden(entry: &DirEntry) -> bool {
        entry.file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }

    #[cfg(test)]
    mod tests {

        use super::*;

        #[test]
        fn test_locate_file() {
            let result = locate_file("main");

            assert!(result.is_some());
        }
    }
}

#[macro_export]
macro_rules! vprintln {
    ($($arg:tt)*) => {{
        if ::utils::print::is_verbose() {
            println!($($arg)*);
        }
    }}
}

#[allow(dead_code)]
pub mod print {
    use std::fmt;

    /// Function version of the vprintln macro. Does not support formatting 
    /// (use format! within it).
    pub fn vprint(input: &str) {
        if is_verbose() {
            println!("{}", input);
        }
    }

    pub fn is_verbose() -> bool {
        unsafe {
            ::VERBOSE
        }
    }

    macro_rules! define_colors {
        ($($(#[$attrs:meta])* pub struct $s:ident => $color:expr;)*) => {
            $(
                $(#[$attrs])*
                pub struct $s<T>(pub T);

                impl<T: fmt::Display> fmt::Display for $s<T> {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "\x1b[{}m{}\x1b[0m", $color, self.0)
                    }
                }

                impl<T: fmt::Debug> fmt::Debug for $s<T> {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "\x1b[{}m{:?}\x1b[0m", $color, self.0)
                    }
                }
            )*
        }
    }

    define_colors! {
        /// Write a `Display` or `Debug` escaped with Red
        pub struct Red => "31";

        /// Write a `Display` or `Debug` escaped with Green
        pub struct Green => "32";

        /// Write a `Display` or `Debug` escaped with Yellow
        pub struct Yellow => "33";
    }
}
