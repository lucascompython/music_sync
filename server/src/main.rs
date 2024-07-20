use std::collections::HashSet;
use std::fs;
use std::io;
use std::sync::Arc;
use std::sync::RwLock;

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use utils::{cbf, encryption::TokenVerifier, split_strings::SplitStrings};

struct AppState {
    file_names: HashSet<String>,
    file_entries: Vec<cbf::FileEntry>,
    config: Config,
    token_verifier: TokenVerifier,
}

struct Config {
    token: String,
    music_dir: String,
    port: u16,
}

impl Config {
    fn new() -> io::Result<Config> {
        let buffer = if let Ok(buffer) = fs::read_to_string("config.conf") {
            buffer
        } else {
            fs::write("config.conf", "token_here\nmusic_dir_path_here")?;
            eprintln!("Please fill out the config.conf file and run the program again");
            std::process::exit(1);
        };

        let mut lines = buffer.lines();

        let token = lines.next().expect("Missing token").to_string();
        let music_dir = lines.next().expect("Missing music_dir").to_string();
        let port = lines
            .next()
            .expect("Missing port")
            .parse()
            .expect("Invalid port");

        Ok(Self {
            token,
            music_dir,
            port,
        })
    }
}

fn validate_token(req: &HttpRequest, token_verifier: &TokenVerifier) -> bool {
    req.headers()
        .get("Authorization")
        .and_then(|header_value| {
            header_value.to_str().ok().and_then(|token| {
                token_verifier
                    .decrypt(token)
                    .filter(|decrypted_token| token_verifier.verify(decrypted_token))
            })
        })
        .is_some()
}

#[get("/sync")]
async fn sync_get(
    state: web::Data<Arc<RwLock<AppState>>>,
    req_body: String,
    req: HttpRequest,
) -> impl Responder {
    let state = state.read().unwrap();

    if !validate_token(&req, &state.token_verifier) {
        return HttpResponse::Unauthorized().finish();
    }

    let files = &state.file_names;
    let incoming_files: HashSet<String> = SplitStrings::new(&req_body, '|').collect();
    let missing: HashSet<&String> = incoming_files.difference(files).collect(); // files that are in the client's request but not in the server's files
    let extra: HashSet<&String> = files.difference(&incoming_files).collect(); // files that are in the server's files but not in the client's request

    if !extra.is_empty() {
        let extra_files: Vec<&cbf::FileEntry> = state
            .file_entries
            .iter()
            .filter(|entry| extra.contains(&entry.name))
            .collect();

        let mut buffer = Vec::new();
        cbf::write(&mut buffer, &extra_files, Some(&missing)).unwrap();

        return HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(buffer);
    }
    if !missing.is_empty() {
        let response = utils::join_hashset(&missing, '|');

        return HttpResponse::Ok().body(response);
    }

    HttpResponse::Ok().body("synced")
}

#[post("/sync")]
async fn sync_post(
    state: web::Data<Arc<RwLock<AppState>>>,
    req_body: web::Bytes,
    req: HttpRequest,
) -> impl Responder {
    let mut state = state.write().unwrap();

    if !validate_token(&req, &state.token_verifier) {
        return HttpResponse::Unauthorized().finish();
    }

    let mut cursor = std::io::Cursor::new(req_body.as_ref());

    let (_, entries) = cbf::read(&mut cursor).unwrap();

    for entry in entries {
        fs::write(
            format!("{}/{}", state.config.music_dir, entry.name),
            &entry.data,
        )
        .unwrap();

        state.file_names.insert(entry.name.clone());
        state.file_entries.push(entry);
    }

    HttpResponse::Ok().body("synced")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let config = Config::new()?;
    let port = config.port;

    println!("Starting server on 0.0.0.0:{}!", port);

    let token_verifier = TokenVerifier::new(&config.token);

    let (file_names, file_entries) = utils::get_files(&config.music_dir)?;

    let state = Arc::new(RwLock::new(AppState {
        file_names,
        file_entries,
        config,
        token_verifier,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 1024 * 10)) // 10 GB
            .service(sync_get)
            .service(sync_post)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
