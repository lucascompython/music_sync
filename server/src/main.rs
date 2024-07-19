use std::collections::HashSet;
use std::fs;
use std::io;
use std::sync::Arc;
use std::sync::RwLock;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use utils::cbf;
use utils::split_strings::SplitStrings;

fn get_file_names_and_get_cbf() -> io::Result<(HashSet<String>, Vec<u8>)> {
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

    cbf::write(&mut buffer, &entries)?;

    Ok((file_names, buffer))
}

#[get("/compare")]
async fn compare(
    files: web::Data<Arc<RwLock<HashSet<String>>>>,
    req_body: String,
) -> impl Responder {
    let files = files.read().unwrap();
    println!("{:?}", files);

    if req_body.is_empty() {
        return HttpResponse::BadRequest().body("No files provided");
    }

    let incoming_files: HashSet<String> = SplitStrings::new(&req_body, '|').collect();

    let missing: HashSet<&String> = incoming_files.difference(&files).collect(); // files that are in the client's request but not in the server's files
    let extra: HashSet<&String> = files.difference(&incoming_files).collect(); // files that are in the server's files but not in the client's request

    let mut response = String::new();

    if !missing.is_empty() {
        response.push_str("Missing files:\n");
        for file in &missing {
            response.push_str(&format!("  {}\n", file));
        }
    }

    if !extra.is_empty() {
        response.push_str("Extra files:\n");
        for file in &extra {
            response.push_str(&format!("  {}\n", file));
        }
    }

    HttpResponse::Ok().body(response)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("Starting server!");

    let (file_names, buffer) = get_file_names_and_get_cbf()?;
    let file_names = Arc::new(RwLock::new(file_names));
    fs::write("glob.cbf", buffer)?;

    println!("{:?}", file_names.read().unwrap());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(file_names.clone()))
            .service(compare)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
