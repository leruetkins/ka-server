use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;


#[derive(Clone)]
struct AppState {
    mp3_files: Vec<PathBuf>,
    current_file_index: usize,
}

#[get("/file.mp3")]
async fn serve_mp3_file(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let mut state = data.lock().await;
    let current_file_index = state.current_file_index;
    let mp3_files = &state.mp3_files;

    if mp3_files.is_empty() {
        return HttpResponse::NotFound().body("No MP3 files found");
    }

    let file_path = &mp3_files[current_file_index];
    let file_name = file_path.file_name().unwrap_or_default();
    let mut file = match File::open(file_path).await {
        Ok(file) => file,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let mut contents = Vec::new();
    if let Err(_) = file.read_to_end(&mut contents).await {
        return HttpResponse::InternalServerError().finish();
    }

    println!("Sending file: {:?}", file_name);

    let response = HttpResponse::Ok()
    .content_type("audio/mpeg")
    .append_header(("Content-Disposition", format!(r#"attachment; filename="{}""#, file_name.to_string_lossy())))
    .body(contents);

    state.current_file_index = (current_file_index + 1) % mp3_files.len();

    response
}

#[get("/file.m3u")]
async fn serve_m3u_playlist(req: actix_web::HttpRequest) -> impl Responder {
    let mp3_files = scan_directory(Path::new("./mp3-files")).unwrap();

    if mp3_files.is_empty() {
        return HttpResponse::NotFound().body("No MP3 files found");
    }

    let host = req.headers()
        .get("host")
        .and_then(|h| {
            let full_host = h.to_str().ok()?;
            if let Some((ip, _)) = full_host.rsplit_once(':') {
                Some(ip)
            } else {
                Some(full_host)
            }
        })
        .unwrap_or_default();  
    
    println!("{}!",host);

    let playlist = format!("#EXTM3U\n#EXTINF:-1,File\nhttp://{}:3000/file.mp3", host);

    println!("Sending M3U");

    HttpResponse::Ok()
        .content_type("audio/mpegurl")
        .append_header(("Content-Disposition", format!(r#"attachment; filename="file.m3u""#)))
        .body(playlist)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mp3_files = scan_directory(Path::new("./mp3-files")).unwrap();

    let state = web::Data::new(Mutex::new(AppState {
        mp3_files,
        current_file_index: 0,
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(serve_mp3_file)
            .service(serve_m3u_playlist)
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}



fn scan_directory(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut mp3_files = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_dir() {
            mp3_files.extend(scan_directory(&file_path)?);
        } else if let Some(extension) = file_path.extension() {
            if extension.to_string_lossy().to_lowercase() == "mp3" {
                mp3_files.push(file_path.to_owned());
            }
        }
    }

    for file_path in &mp3_files {
        println!("Found MP3 file: {:?}", file_path);
    }

    Ok(mp3_files)
}

