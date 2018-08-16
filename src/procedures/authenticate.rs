use std;
use std::io::{stdout, Write};
use raze::engine::engine::Raze;
use raze;

/// Authenticates a raze instance
///
/// This uses the users account id and API key to authenticate \
/// This will attempt to read from a file. If the file is not found, it will prompt for this information.
/// If the user supplies this information and the authentication succeeds, the auth information will be stored
pub fn auth(raze: &mut Raze){
    match raze.authenticate_from_file(std::path::Path::new(::CREDENTIALS_FILE_NAME)) {
        Some(e) => {
            println!("Failed to authenticate with credentials file");
            match e {
                raze::B2Error::B2Error(x) => println!("Server response: {}", x.message),
                x => println!("Unexpected error: {:?}", x),
            }
            println!("Please manually enter authentication");
            println!("Your account id and API key can be found via the website");
            print!("Account id: ");
            stdout().flush().unwrap();
            let account_id: String = read!("{}\n");
            print!("API key: ");
            stdout().flush().unwrap();
            let api_key: String = read!("{}\n");
            let auth = format!("{}:{}", account_id, api_key);
            match raze.authenticate(&auth) {
                Some(e) => {
                    println!("Authentication failure!");
                    println!("{:?}", e);
                    std::process::exit(0);
                }
                _ => {
                    println!("Successfully authenticated, credentials stored file: '{}' ", ::CREDENTIALS_FILE_NAME);
                    let mut cred_file = std::fs::File::create(std::path::Path::new(::CREDENTIALS_FILE_NAME)).unwrap();
                    cred_file.write_all(auth.as_bytes()).unwrap();
                },
            }
        },
        None => println!("Successfully authenticated")
    }
}