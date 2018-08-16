extern crate raze;
#[macro_use] extern crate text_io;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate glob;
extern crate progress;
extern crate sha1;
extern crate scoped_pool;

use raze::engine::engine;
use std::io::Write;

mod storage;
use storage::storage as storage_helper;
mod formatting;
use formatting::time_formatter::*;
mod procedures;


// Name of the file containing program info/options
const PERSISTENT_DATA_FILE_NAME: &str = &"backupdata";
// Name of the file containing the list of folders to backup
const BACKUP_LIST_FILE_NAME: &str = &"backuplist";
// Name of the file containing credentials
const CREDENTIALS_FILE_NAME: &str = &"raze_credentials";
// After this many bytes, switch to upload_file_streaming to reduce memory usage
const STREAM_UPLOAD_THRESHOLD: u64 = 5*1000*1000;
// The amount of simultaneous uploads
const UPLOAD_THREADS: usize = 4;

fn main() {
    println!("Raze CLI - {}", env!("CARGO_PKG_VERSION"));
    // First off, create the backuplist file if it doesn't exist
    if !std::path::Path::new(BACKUP_LIST_FILE_NAME).exists() {
        let mut write = std::fs::File::create(std::path::Path::new(BACKUP_LIST_FILE_NAME)).unwrap();
        write.write_all("Absolute paths of directories to back up goes here\n\
        eg. /home/MyUser/Documents".as_bytes()).unwrap();
    }
    // Then make sure a credentials file exists
    if !std::path::Path::new(CREDENTIALS_FILE_NAME).exists() {
        let mut write = std::fs::File::create(std::path::Path::new(CREDENTIALS_FILE_NAME)).unwrap();
        write.write_all("xxxxxxxxxxxx:yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy".as_bytes()).unwrap();
    }


    let mut raze = engine::Raze::new();
    println!("Authenticating...");
    procedures::authenticate::auth(&mut raze);

    // Find out when the last backup was performed, if ever
    let mut persistent_data = match storage_helper::PersistentData::from_file(&std::path::Path::new(PERSISTENT_DATA_FILE_NAME)) {
        Ok(v) => {
            println!("It has been {} since the last successful backup", time_since_timestamp(v.last_backup));
            v
        },
        Err(_e) => {
            println!("No successful backups have been made yet!");
            storage_helper::PersistentData {
                last_backup: time::get_time().sec,
                active_bucket: String::new(),
                bandwidth_limit: 0,
            }
        },
    };
    persistent_data.save_to_file(&std::path::Path::new(PERSISTENT_DATA_FILE_NAME));

    println!("Type 'help' for a list of commands");
    // Continuously ask for commands, until the program exits
    loop {
        procedures::command_prompt::command_prompt(&mut raze, &mut persistent_data);
    }
}