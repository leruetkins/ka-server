use actix_web::{ get, web, App, HttpResponse, HttpServer, Responder, Result };
use std::fs;
use std::path::{ Path, PathBuf };
use tokio::io::AsyncReadExt;
use serde::Serialize;
use actix_files::NamedFile;
use std::collections::VecDeque;

use urlencoding::decode;
use actix_web::HttpRequest;

use std::sync::Arc;
use std::sync::RwLock;

use std::collections::HashMap;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
struct FolderTree {
    root: FolderNode,
}

#[derive(Serialize)]
struct FolderNode {
    path: String,
    folders: Vec<FolderNode>,
}

#[get("/")]
async fn show_folder_tree() -> actix_web::Result<actix_web::HttpResponse> {
    let folder_tree = scan_directory_tree(std::path::Path::new("./mp3-files"))?;
    let html = generate_html(&folder_tree);
    Ok(actix_web::HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html))
}

fn scan_directory_tree(directory: &std::path::Path) -> actix_web::Result<FolderNode> {
    let folder_name = directory.file_name().unwrap_or_default().to_string_lossy().to_string();
    let mut folders = Vec::new();

    for entry in std::fs
        ::read_dir(directory)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))? {
        let entry = entry.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        let file_path = entry.path();

        if file_path.is_dir() {
            let subfolder = scan_directory_tree(&file_path)?;
            folders.push(subfolder);
        }
    }

    Ok(FolderNode {
        path: folder_name,
        folders,
    })
}

fn generate_html(folder_tree: &FolderNode) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>");
    html.push_str("<html>");

    html.push_str("<head>");
    html.push_str("<meta charset='UTF-8'>");
    html.push_str("<meta name='viewport' content='width=device-width, initial-scale=1'>");
    html.push_str("<title>Media Server</title>");

    html.push_str("<style>");
    html.push_str("@media (max-width: 768px) {");
    html.push_str("  .flex-container {");
    html.push_str("    flex-direction: column;");
    html.push_str("  }");
    html.push_str("}");
    html.push_str("</style>");

    html.push_str("</head>");

    html.push_str("<body>");

    html.push_str("<div class='flex-container'>");

    html.push_str("<div class='folder-tree'>");
    html.push_str("<ul>");
    html.push_str(&generate_folder_node_html(folder_tree, ""));
    html.push_str("</ul>");
    html.push_str("</div>");

    html.push_str("<div class='file-list'>");
    // здесь вывод списка файлов
    html.push_str("</div>");

    html.push_str("</div>");

    html.push_str("</body>");
    html.push_str("</html>");

    html
}

fn generate_folder_node_html(folder_node: &FolderNode, parent_path: &str) -> String {
    let mut html = String::new();
    let folder_path = format!("{}/{}", parent_path, folder_node.path);

    html.push_str("<li>");

    let folder_path_with_extension = format!("{}.radio.m3u", folder_path);

    html.push_str(&format!("<a href='{}'>{}</a>", folder_path_with_extension, folder_node.path));

    if !folder_node.folders.is_empty() {
        html.push_str("<ul>");
        for subfolder in &folder_node.folders {
            html.push_str(&generate_folder_node_html(subfolder, &folder_path));
        }
        html.push_str("</ul>");
    }

    html.push_str("</li>");

    html
}

// #[get("/file.mp3")]
// async fn serve_mp3_file(data: web::Data<Mutex<AppState>>) -> impl Responder {
//     let mut state = data.lock().await;
//     let current_file_index = state.current_file_index;
//     let mp3_files = &state.mp3_files;

//     if mp3_files.is_empty() {
//         return HttpResponse::NotFound().body("No MP3 files found");
//     }

//     let file_path = &mp3_files[current_file_index];
//     let file_name = file_path.file_name().unwrap_or_default();
//     let mut file = match File::open(file_path).await {
//         Ok(file) => file,
//         Err(_) => {
//             return HttpResponse::InternalServerError().finish();
//         }
//     };

//     let mut contents = Vec::new();
//     if let Err(_) = file.read_to_end(&mut contents).await {
//         return HttpResponse::InternalServerError().finish();
//     }

//     println!("Sending file: {:?}", file_name);

//     let response = HttpResponse::Ok()
//         .content_type("audio/mpeg")
//         .append_header((
//             "Content-Disposition",
//             format!(r#"attachment; filename="{}""#, file_name.to_string_lossy()),
//         ))
//         .append_header(("icy-name", format!(r#"Radio-NP - ${}"#, file_name.to_string_lossy())))
//         .append_header(("icy-description", format!(r#"Radio-NP"#)))
//         .body(contents);

//     state.current_file_index = (current_file_index + 1) % mp3_files.len();

//     response
// }

// #[get("/file.m3u")]
// async fn serve_m3u_playlist(req: actix_web::HttpRequest) -> impl Responder {
//     let mp3_files = match scan_directory(Path::new("./mp3-files")) {
//         Ok(files) => files,
//         Err(_) => {
//             return HttpResponse::InternalServerError().body("Failed to scan directory");
//         }
//     };

//     if mp3_files.is_empty() {
//         return HttpResponse::NotFound().body("No MP3 files found");
//     }

//     let host = req
//         .headers()
//         .get("host")
//         .and_then(|h| {
//             let full_host = h.to_str().ok()?;
//             if let Some((ip, _)) = full_host.rsplit_once(':') {
//                 Some(ip)
//             } else {
//                 Some(full_host)
//             }
//         })
//         .unwrap_or_default();

//     println!("{}!", host);

//     let playlist = format!("#EXTM3U\n#EXTINF:-1,File\nhttp://{}:3000/file.mp3", host);

//     println!("Sending M3U");

//     HttpResponse::Ok()
//         .content_type("audio/mpegurl")
//         .append_header(("Content-Disposition", format!(r#"attachment; filename="file.m3u""#)))
//         .body(playlist)
// }

#[get("/{path:.+}.m3u")]
async fn show_m3u(path: web::Path<PathBuf>) -> Result<NamedFile> {
    println!("Playlist");
    let requested_path = path.to_string_lossy().into_owned();
    println!("Requested Path: {}", requested_path);

    let path_without_extension = requested_path.trim_end_matches(".m3u");
    println!("path_without_extension : {}", path_without_extension);
    let directory_path = std::path::Path::new(path_without_extension);
    println!("Directory Path: {:?}", directory_path);

    if directory_path.exists() {
        let mp3_files = scan_directory_files(&directory_path)?;
        let playlist = generate_m3u_playlist(&mp3_files);
        let file_path = std::path::PathBuf::from("temp.m3u");
        std::fs::write(&file_path, playlist)?;

        Ok(NamedFile::open(file_path)?)
    } else {
        Err(actix_web::error::ErrorNotFound("Path not found"))
    }
}

#[get("/{path:.+}.mp3")]
async fn download_mp3_file(req: HttpRequest) -> Result<NamedFile> {
    let full_path = req.path();
    let decoded_path = decode(&full_path[1..]).unwrap();
    let file_path = Path::new(".").join(decoded_path.as_ref());

    println!("File Path: {:?}", file_path); // Print the file_path for debugging

    if file_path.exists() && file_path.is_file() {
        Ok(NamedFile::open(file_path)?)
    } else {
        Err(actix_web::error::ErrorNotFound("File not found"))
    }
}

#[derive(Default)]
struct AppState {
    client_data: Arc<RwLock<HashMap<String, ClientData>>>,
}

#[derive(Default)]
struct ClientData {
    mp3_files: Vec<PathBuf>,
    current_file_index: usize,
}

async fn scan_directory(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut mp3_files = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back(directory.to_owned());

    while let Some(dir) = queue.pop_front() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_dir() {
                queue.push_back(file_path);
            } else if let Some(extension) = file_path.extension() {
                if extension.to_string_lossy().to_lowercase() == "mp3" {
                    mp3_files.push(file_path.to_owned());
                }
            }
        }
    }

    mp3_files.sort();
    println!("Scanned MP3 files:");
    for file_path in &mp3_files {
        // Convert the path to a string and replace backslashes with forward slashes
        let normalized_path = file_path.to_string_lossy().replace("\\", "/");
        println!("{}", normalized_path);
    }

    Ok(mp3_files)
}

#[get("/{path:.+}.radio.m3u")]
async fn show_radio_m3u(
    req: actix_web::HttpRequest,
    path: web::Path<(String,)>,
    data: web::Data<AppState>
) -> impl Responder {
    let file_path = format!("{}", path.into_inner().0);
    let full_path = format!("./{}", file_path);
    println!("Radio Playlist");
    println!("File Path: {}", full_path); // Вывод пути файла для отладки

    let mp3_files = scan_directory(Path::new(&full_path)).await.unwrap();

    // Получение IP-адреса клиента
    let remote_addr = req.connection_info().peer_addr().unwrap_or("Unknown").to_owned();

    // Получение клиентских данных
    let mut client_data = data.client_data.write().unwrap();
    let data = client_data.entry(remote_addr.clone()).or_insert_with(Default::default);

    // Обновление клиентских данных
    data.mp3_files = mp3_files.clone();

    if mp3_files.is_empty() {
        return HttpResponse::NotFound().body("No MP3 files found");
    }

    let host = req
        .headers()
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

    // println!("{}!", host);

    // Get user agent header and convert it to a string
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .unwrap_or("Unknown User Agent");

    println!("User Agent: {}", user_agent);

    let playlist = format!(
        "#EXTM3U\n#EXTINF:-1,File\nhttp://{}:3000/{}.radio.mp3",
        host,
        file_path
    );

    println!("Sending M3U to client: {}", remote_addr); // Отображение IP-адреса клиента

    HttpResponse::Ok()
        .content_type("audio/mpegurl")
        .append_header(("Content-Disposition", format!(r#"attachment; filename="file.m3u""#)))
        .append_header(("X-Client-ID", remote_addr.clone())) // Добавление IP-адреса клиента в заголовки ответа
        .body(playlist)
}

#[get("/{path:.+}.radio.mp3")]
async fn show_radio_mp3(req: actix_web::HttpRequest, data: web::Data<AppState>) -> impl Responder {
    println!("Radio mp3");

    // Получение IP-адреса клиента
    let remote_addr = req.connection_info().peer_addr().unwrap_or("Unknown").to_owned();

    // Получение клиентских данных
    let mut client_data = data.client_data.write().unwrap();
    let data = match client_data.get_mut(&remote_addr) {
        Some(data) => data,
        None => {
            return HttpResponse::NotFound().body("No client data found");
        }
    };

    let mp3_files = &data.mp3_files;
    let current_file_index = &mut data.current_file_index;

    if current_file_index > &mut mp3_files.len() {
        *current_file_index = 0;
    }

    println!("Current file index: {}", current_file_index);

    if mp3_files.is_empty() {
        return HttpResponse::NotFound().body("No MP3 files found");
    }

    let file_path = &mp3_files[*current_file_index];
    let file_name = file_path.file_name().unwrap_or_default();
    let mut file = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut contents = Vec::new();
    if let Err(_) = file.read_to_end(&mut contents).await {
        return HttpResponse::InternalServerError().finish();
    }

    println!("Sending file: {:?} to client: {}", file_name, remote_addr);

    let response = HttpResponse::Ok()
        .content_type("audio/mpeg")
        .append_header((
            "Content-Disposition",
            format!(r#"attachment; filename="{}""#, file_name.to_string_lossy()),
        ))
        .append_header(("icy-name", format!(r#"Radio-NP - {}"#, file_name.to_string_lossy())))
        .append_header(("icy-description", format!(r#"Radio-NP"#)))
        .append_header(("X-Client-ID", remote_addr.clone())) // Добавление IP-адреса клиента в заголовки ответа
        .body(contents);

    *current_file_index = (*current_file_index + 1) % mp3_files.len();

    response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("ka-server {}. ©All rights in reserve.", APP_VERSION);
    let app_state = web::Data::new(AppState {
        client_data: Arc::new(RwLock::new(HashMap::new())),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(show_radio_m3u)
            .service(show_radio_mp3)
            .service(show_folder_tree)
    })
        .bind("0.0.0.0:3000")?
        .run().await
}

fn generate_m3u_playlist(mp3_files: &[std::path::PathBuf]) -> String {
    let mut playlist = String::new();

    for file_path in mp3_files {
        playlist.push_str(&format!("{}\n", file_path.display()));
    }

    playlist
}

fn scan_directory_files(directory: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut mp3_files = Vec::new();

    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_dir() {
            mp3_files.extend(scan_directory_files(&file_path)?);
        } else if let Some(extension) = file_path.extension() {
            if extension.to_string_lossy().to_lowercase() == "mp3" {
                mp3_files.push(file_path.to_owned());
            }
        }
    }

    Ok(mp3_files)
}
