use std::collections::HashSet;
use std::fs;
use std::io;
use std::sync::Arc;
use std::sync::RwLock;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use utils::cbf;
use utils::split_strings::SplitStrings;

struct AppState {
    file_names: HashSet<String>,
    file_entries: Vec<cbf::FileEntry>,
}

#[get("/sync")]
async fn sync_get(state: web::Data<Arc<RwLock<AppState>>>, req_body: String) -> impl Responder {
    let state = state.read().unwrap();
    let files = &state.file_names;

    if req_body.is_empty() {
        let mut buffer = Vec::new();
        cbf::write(&mut buffer, &state.file_entries, None::<&HashSet<String>>).unwrap();

        return HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(buffer);
    }

    let incoming_files: HashSet<String> = SplitStrings::new(&req_body, '|').collect();

    let missing: HashSet<&String> = incoming_files.difference(&files).collect(); // files that are in the client's request but not in the server's files
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
) -> impl Responder {
    let mut cursor = std::io::Cursor::new(req_body.as_ref());

    let (_, entries) = cbf::read(&mut cursor).unwrap();

    let mut state = state.write().unwrap();
    for entry in entries {
        fs::write(format!("music/{}", entry.name), &entry.data).unwrap();
        state.file_names.insert(entry.name.clone());
        state.file_entries.push(entry);
    }

    HttpResponse::Ok().body("synced")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("Starting server!");

    let (file_names, file_entries) = utils::get_files()?;

    let state = Arc::new(RwLock::new(AppState {
        file_names,
        file_entries,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 1024 * 10)) // 10 GB
            .service(sync_get)
            .service(sync_post)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
