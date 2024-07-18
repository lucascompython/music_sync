use std::fs;
use std::io;
use std::io::Write;
use std::sync::Arc;
use std::sync::RwLock;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

fn get_file_names_and_get_cbf() -> io::Result<(Vec<String>, Vec<u8>)> {
    let path = fs::read_dir("music")?;

    let mut entries = Vec::new();
    let mut file_names = Vec::new();
    for path in path {
        let path = path?.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        file_names.push(file_name.clone());

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
async fn compare(files: web::Data<Arc<RwLock<Vec<String>>>>, req_body: String) -> impl Responder {
    let files = files.read().unwrap();
    println!("{:?}", files);

    if req_body.is_empty() {
        return HttpResponse::BadRequest().body("No files provided");
    }

    let incoming_files: Vec<&str> = req_body.split("|").collect();
    let incoming_files_len = incoming_files.len();

    let mut missing = Vec::new();

    for (i, file) in files.iter().enumerate() {
        if i >= incoming_files_len {
            missing.push(file);
            continue;
        }

        unsafe {
            if file != incoming_files.get_unchecked(i) {
                missing.push(file);
            }
        }
    }

    HttpResponse::Ok().body(req_body)
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
