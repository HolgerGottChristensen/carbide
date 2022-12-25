use std::{env, fs};
use std::io;
use std::path::{Path, PathBuf};

/// This file originated from https://github.com/PistonDevelopers/find_folder/blob/master/src/lib.rs
/// Updated to not use deprecated try macro

/// Depth of recursion through kids.
pub type KidsDepth = u8;
/// Depth of recursion through parents.
pub type ParentsDepth = u8;

/// The direction in which `find_folder` should search for the folder.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Search {
    /// Search recursively through parent directories with the given depth.
    Parents(ParentsDepth),
    /// Search recursively through children directories with the given depth.
    Kids(KidsDepth),
    /// Search parents and then kids (same as `Both`).
    ParentsThenKids(ParentsDepth, KidsDepth),
    /// Search kids and then parents.
    KidsThenParents(KidsDepth, ParentsDepth),
}

/// A search defined as a starting path and a route to take.
///
/// Don't instantiate this type directly. Instead, use `Search::of`.
#[derive(Clone, Debug)]
pub struct SearchFolder {
    /// The starting path of the search.
    pub start: PathBuf,
    /// The route to take while searching.
    pub direction: Search,
}

/// If the search was unsuccessful.
#[derive(Debug)]
pub enum Error {
    /// Some std io Error occurred.
    IO(::std::io::Error),
    /// The directory requested was not found.
    NotFound,
}

impl ::std::convert::From<io::Error> for Error {
    fn from(io_err: io::Error) -> Error {
        Error::IO(io_err)
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        writeln!(f, "{:?}", *self)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(ref io_err) => ::std::error::Error::description(io_err),
            Error::NotFound => "The folder could not be found",
        }
    }
}

impl Search {
    /// An easy API method for finding a folder with a given name.
    /// i.e. `Search::Kids(u8).for_folder("assets")`
    pub fn for_folder(&self, target: &str) -> Result<PathBuf, Error> {
        let cwd = env::current_dir()?;
        self.of(cwd).for_folder(target)
    }

    /// Use this to search in a specific folder.
    ///
    /// This method transforms a `Search` into a `SearchFolder`, but that detail is mostly
    /// irrelevant. See the example for recommended usage.
    ///
    /// # Example
    ///
    /// ```
    /// use find_folder::Search;
    ///
    /// let mut exe_folder = std::env::current_exe().unwrap();
    /// exe_folder.pop(); // Remove the executable's name, leaving the path to the containing folder
    /// let resource_path = Search::KidsThenParents(1, 2).of(exe_folder).for_folder("resources");
    /// ```
    pub fn of(self, start: PathBuf) -> SearchFolder {
        SearchFolder {
            start: start,
            direction: self,
        }
    }
}

impl SearchFolder {
    /// Search for a folder with the given name.
    pub fn for_folder(&self, target: &str) -> Result<PathBuf, Error> {
        match self.direction {
            Search::Parents(depth) => check_parents(depth, target, &self.start),
            Search::Kids(depth) => check_kids(depth, target, &self.start),
            Search::ParentsThenKids(parents_depth, kids_depth) => {
                match check_parents(parents_depth, target, &self.start) {
                    Err(Error::NotFound) => check_kids(kids_depth, target, &self.start),
                    other_result => other_result,
                }
            }
            Search::KidsThenParents(kids_depth, parents_depth) => {
                match check_kids(kids_depth, target, &self.start) {
                    Err(Error::NotFound) => check_parents(parents_depth, target, &self.start),
                    other_result => other_result,
                }
            }
        }
    }
}

/// Check the contents of this folder and children folders.
pub fn check_kids(depth: u8, name: &str, path: &Path) -> Result<PathBuf, Error> {
    match check_dir(name, path) {
        err @ Err(Error::NotFound) => match depth > 0 {
            true => {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if fs::metadata(&entry_path)?.is_dir() {
                        if let Ok(folder) = check_kids(depth - 1, name, &entry_path) {
                            return Ok(folder);
                        }
                    }
                }
                err
            }
            false => err,
        },
        other_result => other_result,
    }
}

/// Check the given path and `depth` number of parent directories for a folder with the given name.
pub fn check_parents(depth: u8, name: &str, path: &Path) -> Result<PathBuf, Error> {
    match check_dir(name, path) {
        err @ Err(Error::NotFound) => match depth > 0 {
            true => match path.parent() {
                None => err,
                Some(parent) => check_parents(depth - 1, name, parent),
            },
            false => err,
        },
        other_result => other_result,
    }
}

/// Check the given directory for a folder with the matching name.
pub fn check_dir(name: &str, path: &Path) -> Result<PathBuf, Error> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.ends_with(name) {
            return Ok(entry_path);
        }
    }
    Err(Error::NotFound)
}
