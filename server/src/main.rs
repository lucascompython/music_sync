use std::collections::HashSet;
use std::fs;
use std::io;
use std::sync::Arc;
use std::sync::RwLock;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use utils::cbf;
use utils::split_strings::SplitStrings;

struct AppState {
    file_names: HashSet<String>,
    file_entries: Vec<cbf::FileEntry>,
}

fn get_file_names_and_get_cbf() -> io::Result<(HashSet<String>, Vec<u8>, Vec<cbf::FileEntry>)> {
    let path = if let Ok(path) = fs::read_dir("music") {
        path
    } else {
        fs::create_dir("music")?;
        fs::read_dir("music")?
    };

    let mut entries = Vec::new();
    let mut file_names = HashSet::new();
    for path in path {
        let path = path?.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        file_names.insert(file_name.clone());

        let data = fs::read(path)?;
        entries.push(cbf::FileEntry {
            name: file_name,
            data,
        });
    }

    let mut buffer = Vec::new();

    cbf::write(&mut buffer, &entries, None::<&HashSet<String>>)?;

    Ok((file_names, buffer, entries))
}

#[get("/compare")]
async fn compare(state: web::Data<Arc<RwLock<AppState>>>, req_body: String) -> impl Responder {
    let state = state.read().unwrap();
    let files = &state.file_names;
    println!("Server files: {:?}", files);

    if req_body.is_empty() {
        return HttpResponse::BadRequest().body("No files provided");
    }

    let incoming_files: HashSet<String> = SplitStrings::new(&req_body, '|').collect();
    println!("Incoming files: {:?}", &incoming_files);

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

    HttpResponse::Ok().body("yeah")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("Starting server!");

    let (file_names, buffer, file_entries) = get_file_names_and_get_cbf()?;
    fs::write("glob.cbf", buffer)?;

    let state = Arc::new(RwLock::new(AppState {
        file_names,
        file_entries,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(compare)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
