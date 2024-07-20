use std::{
    collections::HashSet,
    fs,
    io::{self, Read},
};
use utils::{cbf, encryption::TokenVerifier, split_strings::SplitStrings};

struct Config {
    server_url: String,
    token: String,
    music_dir: String,
}

impl Config {
    fn new() -> io::Result<Config> {
        let mut file = if let Ok(file) = fs::File::open("config.conf") {
            file
        } else {
            fs::write(
                "config.conf",
                "server_url_here\ntoken_here\nmusic_dir_path_here",
            )?;
            eprintln!("Please fill out the config.conf file and run the program again");
            std::process::exit(1);
        };

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let mut lines = buffer.lines();

        let server_url = lines.next().unwrap().to_string();
        let token = lines.next().unwrap().to_string();
        let music_dir = lines.next().unwrap().to_string();

        let config = Config {
            server_url,
            token,
            music_dir,
        };

        Ok(config)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;

    let token_verifier = TokenVerifier::new(&config.token);
    let encrypted_token = token_verifier.encrypt(config.token.as_bytes());

    let (file_names, file_entries) = utils::get_files(&config.music_dir)?;

    let client = reqwest::blocking::Client::new();

    let response = client
        .get(format!("{}/sync", config.server_url))
        .header("Authorization", &encrypted_token)
        .body(utils::join_hashset(&file_names, '|'))
        .send()?;

    if response.status().is_success() {
        let content_type = response.headers().get("content-type");

        match content_type {
            // Basically, if the content type is application/octet-stream
            Some(_) => {
                let buffer = response.bytes()?;
                let mut cursor = std::io::Cursor::new(buffer);

                let (missing_files, entries) = cbf::read(&mut cursor)?;

                println!("The server is missing {} files", missing_files.len());
                println!("The client is missing {} files", entries.len());

                for entry in entries {
                    fs::write(format!("{}/{}", config.music_dir, entry.name), entry.data)?;
                }

                if !missing_files.is_empty() {
                    let mut buffer = Vec::new();
                    let missing_files: Vec<&cbf::FileEntry> = file_entries
                        .iter()
                        .filter(|entry| missing_files.contains(&entry.name))
                        .collect();
                    cbf::write(&mut buffer, &missing_files, None::<&HashSet<String>>)?;

                    let response = client
                        .post(format!("{}/sync", config.server_url))
                        .header("Authorization", &encrypted_token)
                        .body(buffer)
                        .send()?;

                    if response.status().is_success() {
                        if response.text()? == "synced" {
                            println!("Synced missing files!");
                        } else {
                            eprintln!("Failed to sync missing files!");
                        }
                    }
                }
            }
            None => {
                let response_text = response.text()?;

                match response_text.as_str() {
                    "synced" => {
                        println!("Already Synced!");
                    }

                    _ => {
                        let missing_files_names =
                            SplitStrings::new(&response_text, '|').collect::<HashSet<String>>();

                        println!("The server is missing {} files", missing_files_names.len());

                        let missing_files: Vec<&cbf::FileEntry> = file_entries
                            .iter()
                            .filter(|entry| missing_files_names.contains(&entry.name))
                            .collect();

                        let mut buffer = Vec::new();
                        cbf::write(&mut buffer, &missing_files, None::<&HashSet<String>>)?;

                        let response = client
                            .post(format!("{}/sync", config.server_url))
                            .header("Authorization", &encrypted_token)
                            .body(buffer)
                            .send()?;

                        if response.status().is_success() {
                            if response.text()? == "synced" {
                                println!("Synced missing files!");
                            } else {
                                eprintln!("Failed to sync missing files!");
                            }
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to sync files!");
    }

    Ok(())
}
