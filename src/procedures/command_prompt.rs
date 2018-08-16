use std;
use std::io::{stdout, Write};
use raze::engine::engine::Raze;
use storage::storage::PersistentData;

pub fn command_prompt(raze: &mut Raze, persistent_data: &mut PersistentData){
    print!("Raze>");
    stdout().flush().unwrap();
    let input: String = read!("{}\n");
    match input.to_lowercase().trim_right() {
        "help" => {
            println!("Command List");
            println!("'quit' - Exits this program");
            println!("'backup' - Starts a new backup");
            println!("'throttle' - Allows you to set the maximum bytes sent per second");
            println!("'set_bucket' - Lists available buckets and asks which one to use for backups");
            println!("'usage' - Explains how to use this program")
        }
        "quit" => std::process::exit(0),
        "backup" => {
            ::procedures::backup::perform_backup(raze, persistent_data);
        }
        "throttle" => {
            print!("Enter maximum bytes/sec sent during upload: ");
            stdout().flush().unwrap();
            let read: String = read!("{}\n");
            let amount = match read.parse::<usize>() {
                Ok(n) => match n {
                    n if n > ::UPLOAD_THREADS || n == 0 => n,
                    _ => {
                        println!("Input too low -- defaulting to minimum");
                        ::UPLOAD_THREADS
                    }
                },
                Err(_e) => {
                    println!("Invalid input -- defaulting to no throttling");
                    0
                },
            };
            persistent_data.bandwidth_limit = amount;
            persistent_data.save_to_file(&std::path::Path::new(::PERSISTENT_DATA_FILE_NAME));
        }
        "set_bucket" => {
            println!("Available buckets");
            let buckets = raze.list_buckets().unwrap();
            for bucket in &buckets {
                println!("{} - {}", bucket.bucket_name, bucket.bucket_id);
            }
            print!("Enter bucket name: ");
            stdout().flush().unwrap();
            let name: String = read!("{}\n");
            for bucket in &buckets {
                if name == bucket.bucket_name {
                    persistent_data.active_bucket = bucket.bucket_id.clone();
                    persistent_data.save_to_file(&std::path::Path::new(::PERSISTENT_DATA_FILE_NAME));
                }
            }
        }
        "usage" => {
            println!("Raze User Guide");
            println!("Before you can run a backup, you must use the 'set_bucket' command");
            println!("This will be the bucket your files will be stored in");
            println!("You may want to edit the bucket's settings via the web interface");
            println!();
            println!("Running the 'backup' command will start the backup process");
            println!("Edit the '{}' file to specify files/folders for backup", ::BACKUP_LIST_FILE_NAME);
            println!("All sub-folders will be included when selecting a folder!");
            println!();
            println!("The upload speed can be limited by using the 'throttle' command");
            println!();
            println!("The backup process can be stopped at any time and will continue from where it left off");
            println!("Files can be retrieved via the B2 web interface");
            stdout().flush().unwrap();
        }
        _ => println!("Unknown command - Type 'help' for help")
    }
}