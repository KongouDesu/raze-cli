use std;
use std::io::{stdout, Write};
use raze::engine::engine;
use storage::storage as storage_helper;
use scoped_pool::Pool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use progress;
use time;

pub fn purge_files(raze: &mut engine::Raze ,persistent_data: &mut storage_helper::PersistentData) {
    // Verify that a bucket is selected
    if persistent_data.active_bucket == "" {
        println!("Please set a bucket first with the 'set_bucket' command");
        return
    }
    // Set the active bucket
    raze.set_active_bucket(persistent_data.active_bucket.clone());
    println!("Purging start");
    println!("Note: this will only hide the files in the cloud");
    println!("You can (probably should) configure BackBlaze to delete hidden files after a while");
    println!("Constructing file list");

    // Get a list of files for potential upload
    // Note that we do not intend to upload them, this is a delete function :-)
    let mut file_list = storage_helper::create_file_list(
        storage_helper::read_lines_to_vec(
            std::path::Path::new(::BACKUP_LIST_FILE_NAME)).unwrap());
    for i in 0..file_list.len() {
        let fl = file_list[i].clone();
        let entry_parent = fl.parent();
        let mut prefix = "";
        if entry_parent.is_some() {
            let entry_str = entry_parent.unwrap().to_str().unwrap();
            let fwd = entry_str.find("/");
            let bwd = entry_str.find("\\");
            if fwd.is_some() {
                prefix = &entry_str[fwd.unwrap() + 1..];
            } else if bwd.is_some() {
                prefix = &entry_str[bwd.unwrap() + 1..];
            }
        }
        file_list[i] = std::path::PathBuf::from(format!("{}/{}", prefix, file_list[i].file_name().unwrap().to_str().unwrap()).replace("\\", "/"));
    }

    if file_list.len() == 0 {
        println!("It seems like the {} doesn't exist or contains no entries, aborting", ::BACKUP_LIST_FILE_NAME);
        return
    }

    // Sort it so we can use binary search for finding elements
    file_list.sort();

    // Create a progress bar
    // Wrap progress bar and finished_uploads in an Arc(Mutex)
    // This is needed so each thread can redraw a correct progress bar
    let bar = progress::Bar::new();
    let bar = Arc::new(Mutex::new(bar));
    let delete_amount = Arc::new(Mutex::new(0));
    let finished_deletes = Arc::new(Mutex::new(0));
    let saved_space = Arc::new(Mutex::new(0));

    // Before we start uploading, we should check if the file is also on the server
    // To do this, we retrieve a list of all files on the server,
    // check if one of them has the same path+name as the one we're uploading and
    println!("Discovering deletable files...");
    let stored_file_list = raze.list_all_file_names(&persistent_data.active_bucket, 1000).unwrap();
    let stored_file_count = stored_file_list.len();

    // Create a scoped pool and queue each file in the list for uploading
    let pool = Pool::new(::DELETE_THREADS);
    pool.scoped(|scope| {
        for i in 0..stored_file_count {

            // Create a PathBuf from the StoredFile
            let pb = std::path::PathBuf::from(stored_file_list[i].file_name.clone());

            // If it's found, skip to the next file, if not, queue it for uploading
            let should_delete: bool;
            match file_list.binary_search(&pb) {
                Ok(_) => { // A file with the same path+name exists
                    should_delete = false;
                },
                Err(_e) => { // No matching path+name exists
                    should_delete = true;
                    let mut data = delete_amount.clone();
                    *data.lock().unwrap() += 1;
                    let mut data2 = saved_space.clone();
                    *data2.lock().unwrap() += stored_file_list[i].content_length;
                }
            }
            if !should_delete {
                continue;
            }
            // Clone all the data we pass to the thread
            let entry = stored_file_list[i].clone();
            let mut r = raze.clone();
            let fin_deletes = finished_deletes.clone();

            // Queue the delete request
            // Every file gets a maximum of 5 attempts in case they fail for any reason
            // If the request fails, it'll sleep and retry
            scope.execute(move || {
                for attempts in 0..5 {
                    let res = r.hide_file(entry.file_name.clone());
                    match res {
                        Some(_) => {
                            break
                        },
                        None => {
                            if attempts == 4 {
                                println!();
                                println!("Failed to delete {} after 5 attempts",
                                         format!("{}", entry.file_name));
                            }else{
                                // Sleep for a bit before retrying
                                std::thread::sleep(Duration::from_millis(5000));
                            }
                        }
                    }
                }
                let mut data = fin_deletes.lock().unwrap();
                *data += 1;
            });
        }
        {
            let data_clone = delete_amount.clone();
            let data = data_clone.lock().unwrap();
            let data_clone2 = saved_space.clone();
            let data2 = data_clone2.lock().unwrap();
            println!("Deleting {} files, saving {} of cloud space", *data, ::formatting::size_formatter::format_bytes(*data2));
            stdout().flush().unwrap();
        }

        // If there's nothing to delete, just return
        let data_clone = delete_amount.clone();
        let data = data_clone.lock().unwrap();
        if *data == 0 {
            return;
        }

        // Start the loop that prints the progress bar and checks if we're done yet
        let progress_bar = bar.clone();
        progress_bar.lock().unwrap().set_job_title("Deletion in progress");
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            let data2_clone = finished_deletes.clone();
            let data2 = data2_clone.lock().unwrap();
            progress_bar.lock().unwrap().set_job_title(&format!("Deletion in progress ({}/{})",*data2, *data));
            progress_bar.lock().unwrap().reach_percent((((*data2 as f64) / (*data as f64)) * 100.) as i32);
            if *data == *data2 {
                break;
            }
        }
    });
    println!();
    println!("Backup successfully completed");
    persistent_data.last_backup = time::get_time().sec;
    persistent_data.save_to_file(&std::path::Path::new(::PERSISTENT_DATA_FILE_NAME));
}