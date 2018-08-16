use std;
use std::io::{stdout, Write};
use raze::engine::engine;
use raze;
use formatting::size_formatter::format_bytes;
use storage::storage as storage_helper;
use scoped_pool::Pool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use progress;
use time;

pub fn perform_backup(raze: &mut engine::Raze ,persistent_data: &mut storage_helper::PersistentData) {
    // Verify that a bucket is selected
    if persistent_data.active_bucket == "" {
        println!("Please set a bucket first with the 'set_bucket' command");
        return
    }
    // Set the active bucket
    raze.set_active_bucket(persistent_data.active_bucket.clone());

    // Notify the user that they are throttling the upload
    if persistent_data.bandwidth_limit > 0 {
        println!("! INFO ! Uploading is being throttled to {}/sec", format_bytes(persistent_data.bandwidth_limit as u64));
    }

    // Get a list of files for uploading
    let file_list = storage_helper::create_file_list(
        storage_helper::read_lines_to_vec(
            std::path::Path::new(::BACKUP_LIST_FILE_NAME)).unwrap());
    let file_count = file_list.len(); // mut since it may be decreased if duplicates are discovered
    // Get the total size of those files
    let mut list_size = storage_helper::get_total_size(&file_list); // mut since it may be decreased if duplicates are discovered
    println!("Uploading up to {} ({} bytes) across {} files", format_bytes(list_size), list_size, file_count);

    if list_size == 0 {
        println!("It seems like the {} doesn't exist or contains no entries, aborting", ::BACKUP_LIST_FILE_NAME);
        return
    }

    // Create a progress bar
    // Wrap progress bar and finished_uploads in an Arc(Mutex)
    // This is needed so each thread can redraw a correct progress bar
    let bar = progress::Bar::new();
    let bar = Arc::new(Mutex::new(bar));
    let finished_uploads = Arc::new(Mutex::new(0));

    // Before we start uploading, we should check if the file is also on the server
    // To do this, we retrieve a list of all files on the server,
    // check if one of them has the same path+name as the one we're uploading and
    println!("Synchronizing changes, this may take a bit...");
    let mut stored_file_list = raze.list_all_file_names(&persistent_data.active_bucket, 1000).unwrap();
    // Sort it so we can use binary search for finding elements
    stored_file_list.sort();
    let revised_file_count = Arc::new(Mutex::new(file_count));

    // Create a scoped pool and queue each file in the list for uploading
    let pool = Pool::new(::UPLOAD_THREADS);
    pool.scoped(|scope| {
        for i in 0..file_count {
            // First, convert the Path to a prefix and a filename
            // The prefix will be everything between eg. C:\ and the filename, eg. Users\Kongou
            let entry_parent = file_list[i].parent();
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

            // Construct a StoredFile with the intended name so we can search for it
            let sf = raze::api::files::structs::StoredFile {
                file_name: format!("{}/{}", prefix, file_list[i].file_name().unwrap().to_str().unwrap()).replace("\\", "/"),
                file_id: "".to_owned(),
                account_id: None,
                bucket_id: None,
                content_length: 0,
                content_sha1: "".to_owned(),
                content_type: "".to_owned(),
                action: "".to_owned(),
                upload_timestamp: 0
            };
            // If it's found, skip to the next file, if not, queue it for uploading
            let do_upload: bool;
            match stored_file_list.binary_search(&sf) {
                Ok(v) => { // A file with the same path+name exists
                    // Check if the local file was modified since it was last uploaded
                    let metadata = std::fs::metadata(&file_list[i]).unwrap();
                    let modified_time = match metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH) {
                        Ok(v) => v.as_secs()*1000, // Convert seconds to milliseconds
                        Err(_e) => 0u64
                    };
                    //println!("{} vs {} for {:?}",modified_time,stored_file_list[i].upload_timestamp,&file_list[i]);
                    if modified_time > stored_file_list[v].upload_timestamp {
                        do_upload = true;
                    }else{
                        do_upload = false;
                        let mut data = revised_file_count.clone();
                        *data.lock().unwrap() -= 1;
                        list_size -= metadata.len();
                    }
                },
                Err(_e) => { // No matching path+name exists
                    do_upload = true;
                }
            }
            if !do_upload {
                continue;
            }
            // Clone all the data we pass to the thread
            let entry = file_list[i].clone();
            let mut r = raze.clone();
            let fin_uploads = finished_uploads.clone();
            let bandwidth_limit = persistent_data.bandwidth_limit.clone();

            // Queue the upload tasks
            // Every file gets a maximum of 5 attempts in case they fail for any reason
            // This loop will first decide which upload time to use, then call that upload
            // If the upload fails, it'll sleep and retry
            scope.execute(move || {
                for attempts in 0..5 {
                    // We need to decide which upload type to use
                    // First of all, we check whether or not we're throttling our uploads
                    let result = match bandwidth_limit {
                        // If we're not throttling:
                        // Decide which upload type to use, based on the value of STREAM_UPLOAD_THRESHOLD
                        0 => {
                            match entry.metadata().unwrap().len() {
                                x if x < ::STREAM_UPLOAD_THRESHOLD => r.upload_file(entry.as_ref(), prefix),
                                _ => {
                                    r.upload_file_streaming(entry.as_ref(), prefix)
                                },
                            }
                        },
                        // If we are, use throttled upload.
                        // Each thread gets the same bandwidth, equal to bandwidth/num_threads
                        _ => r.upload_file_throttled(entry.as_ref(), prefix, bandwidth_limit/::UPLOAD_THREADS),
                    };

                    match result {
                        Ok(_v) => break,
                        Err(e) => {
                            if attempts == 4 {
                                println!();
                                println!("Failed to upload {} after 5 attempts",
                                         format!("{}/{}", prefix, entry.file_name().unwrap().to_str().unwrap()).replace("\\", "/"));
                                println!("{:?}", e);
                            }else{
                                // Sleep for a bit before retrying
                                std::thread::sleep(Duration::from_millis(5000));
                            }
                        },
                    }
                }
                let mut data = fin_uploads.lock().unwrap();
                *data += 1;
            });
        }
        let data_clone = revised_file_count.clone();
        let data = data_clone.lock().unwrap();
        println!("Uploading {} ({} bytes) across {} files", format_bytes(list_size), list_size, *data);
        stdout().flush().unwrap();

        // If there's nothing to upload, just return
        if list_size == 0 {
            return;
        }

        // Start the loop that prints the progress bar and checks if we're done yet
        let progress_bar = bar.clone();
        progress_bar.lock().unwrap().set_job_title("Upload in progress");
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            let data2_clone = finished_uploads.clone();
            let data2 = data2_clone.lock().unwrap();
            progress_bar.lock().unwrap().set_job_title(&format!("Upload in progress ({}/{})",*data2, *data));
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