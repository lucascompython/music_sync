use mimalloc::MiMalloc;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Read},
    sync::Arc,
};
use utils::{cbf, encryption::TokenVerifier, split_strings::SplitStrings};

struct Config {
    server_url: String,
    token: String,
    music_dir: String,
}

impl Config {
    fn new() -> io::Result<Self> {
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

        Ok(Self {
            server_url: lines.next().unwrap().to_string(),
            token: lines.next().unwrap().to_string(),
            music_dir: lines.next().unwrap().to_string(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::new()?);

    let token_verifier = TokenVerifier::new(&config.token);
    let encrypted_token = token_verifier.encrypt(config.token.as_bytes());

    let (file_names, file_entries) = utils::get_files(&config.music_dir)?;

    let client = reqwest::blocking::Client::new();

    let response = client
        .get(format!("{}/sync", config.server_url))
        .header("Authorization", &encrypted_token)
        .body(utils::join_hashset(&file_names, '|'))
        .send();

    let response = match response {
        Ok(response) => response,
        Err(err) => {
            eprintln!("Failed to sync files: {}", err);
            return Ok(());
        }
    };

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

                let config_clone = config.clone();

                let network_thead = if !missing_files.is_empty() {
                    Some(std::thread::spawn(move || {
                        sync_missing_files(
                            &client,
                            &config_clone,
                            &encrypted_token,
                            &file_entries,
                            &missing_files,
                        )
                        .expect("Failed to sync missing files!");
                    }))
                } else {
                    None
                };

                entries.into_par_iter().for_each(|(name, data)| {
                    fs::write(format!("{}/{}", config.music_dir, name), data).unwrap();
                });

                if let Some(network_thead) = network_thead {
                    network_thead.join().unwrap();
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

                        sync_missing_files(
                            &client,
                            &config,
                            &encrypted_token,
                            &file_entries,
                            &missing_files_names,
                        )?;
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to sync files!");
    }

    Ok(())
}

fn sync_missing_files(
    client: &reqwest::blocking::Client,
    config: &Config,
    encrypted_token: &str,
    file_entries: &cbf::FileEntries,
    missing_files: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let missing_files = missing_files
        .iter()
        .map(|name| (name, file_entries.get(name).unwrap()))
        .collect::<HashMap<_, _>>();

    let mut buffer = Vec::new();
    cbf::write(&mut buffer, &missing_files, None::<&HashSet<&String>>)?;

    let response = client
        .post(format!("{}/sync", config.server_url))
        .header("Authorization", encrypted_token)
        .body(buffer)
        .send()?;
    if response.status().is_success() && response.text()? == "synced" {
        println!("Synced missing files!");
    } else {
        eprintln!("Failed to sync missing files!");
    }

    Ok(())
}
