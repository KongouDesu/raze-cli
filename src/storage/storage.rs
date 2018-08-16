use std;
use std::io::{Read, Write, BufRead};
use serde_json;
use glob::glob;

#[derive(Deserialize, Serialize, Debug)]
pub struct PersistentData {
    pub last_backup: i64,
    pub active_bucket: String,
    pub bandwidth_limit: usize,
}

pub enum StorageError {
    SerdeError(serde_json::Error),
    IOError(std::io::Error),
}

impl PersistentData {
    pub fn from_file(file: &std::path::Path) -> Result<PersistentData,StorageError> {
        let mut read = match std::fs::File::open(file) {
            Ok(f) => f,
            Err(e) => return Err(StorageError::IOError(e)),
        };
        let mut contents = String::new();
        read.read_to_string(&mut contents).unwrap();
        let persdata: PersistentData = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(e) => return Err(StorageError::SerdeError(e)),
        };
        Ok(persdata)
    }

    pub fn save_to_file(&self, file: &std::path::Path){
        // Serialize the struct
        let json = serde_json::to_string(&self).unwrap();
        // Write it to a file
        let mut write = std::fs::File::create(file).unwrap();
        write.write_all(json.as_bytes()).unwrap();
    }
}

// Given a list of files and directories in ABSOLUTE PATH, as strings, returns a list of
// all files contained in the directories and recursively in subdirectories
pub fn create_file_list(entry_list: Vec<String>) -> Vec<std::path::PathBuf> {
    let mut paths = std::vec::Vec::new();
    for entry in entry_list {
        let p = std::path::Path::new(&entry);
        if p.is_file() {
            paths.push(p.to_owned());
        } else {
            paths.append(&mut glob_directory(&entry));
        }
    }
    paths
}

// Recursively globs a directory, returning a vec of all files found
fn glob_directory(dir: &str) -> Vec<std::path::PathBuf>{
    let mut paths = std::vec::Vec::new();
    for entry in glob(&format!("{}/*",dir)).unwrap() {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    paths.push(path);
                } else if path.is_dir() {
                    paths.append(&mut glob_directory(path.to_str().unwrap()));
                }
            }
            // Debug info if we run glob an unreadable path
            Err(e) => println!("Failed to read {:?}", e),
        }
    }
    paths
}

// Returns the total size of all files in the supplied vector of paths
// Panics if any of the paths aren't a file, use with create_file_list
pub fn get_total_size(paths: &Vec<std::path::PathBuf>) -> u64 {
    paths.into_iter().fold(0, |acc, p| acc + std::fs::metadata(p).unwrap().len())
}

// Given a file path, read all non-whitespace lines to a Vec<String>
pub fn read_lines_to_vec(file_path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
    let mut lines = std::vec::Vec::new();
    let f = match std::fs::File::open(file_path) {
        Ok(v) => v,
        Err(_e) => return Ok(Vec::new()),
    };
    let file = std::io::BufReader::new(&f);
    for line in file.lines() {
        let l = match line {
            Ok(v) => v.to_owned(),
            Err(e) => return Err(e),
        };
        match l.as_ref() {
            "" => (),
            _ => lines.push(l),
        }
    }
    Ok(lines)
}

// Test only works if a backup list file exists and is valid
#[test]
fn test_whatever() {
    let n = read_lines_to_vec(std::path::Path::new("backuplist")).unwrap();
    let h = create_file_list(n);
    let o = get_total_size(&h);
    println!("{}",o);
}