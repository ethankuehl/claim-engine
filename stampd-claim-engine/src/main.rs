use std::{io::Write, string};
use std::env;
use std::net::Ipv4Addr;
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm, json
    },
    Multipart
};
use actix_web::{middleware, web, post, App, Error, HttpResponse, HttpServer, Responder, ResponseError};
use c2pa::ManifestStoreReport;
use futures_util::TryStreamExt as _;
use uuid::Uuid;
use std::fs;
use reqwest::multipart::Part;
use log;
use env_logger;
// use tokio::fs::File;
use std::fs::File;
use std::io::Read;
mod c2pa_func;
mod azure_func;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[post("/api/file_and_manifest")]
async fn file_and_manifest(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> HttpResponse {
    let mut file_path_vec = Vec::new();
    let mut file_path = "".to_string();
    let mut signed_file_path = "".to_string();
    let mut file_name = "".to_string();

        for f in form.files {
            file_name = f.file_name.unwrap();
            file_path = format!("./original/{}", file_name.clone());
            // signed_file_path = format!("./signed/{}", file_name.clone());
            file_path_vec.push(file_path.clone());
            // file_path_vec.push(signed_file_path.clone());
            println!("{}", file_path.clone());
            log::info!("saving to {}", file_path.clone());
            f.file.persist(file_path.clone()).unwrap();
            
        }
    println!("Uploaded file");
    signed_file_path = format!("./signed/{}", file_name.clone());
    println!("{}", signed_file_path);
    file_path_vec.push(signed_file_path.clone());
    let _ = fs::copy(file_path.clone(), signed_file_path.clone());

    let _ = c2pa_func::generate_claim(file_path_vec).await;

    let mut file = File::open(signed_file_path.clone()).unwrap();
    let mut vec = Vec::new(); 
    let _ = file.read_to_end(&mut vec);

    // Gettng file extension
    let file_path_seperator =file_name.split('.');
    let collection: Vec<&str> = file_path_seperator.collect();
    let file_extension = collection.last().unwrap().to_owned();
    let mut content_type = "";
    match file_extension {
        "jpg" => content_type = "image/png",
        "png" => content_type = "image/png",
        "mp3" => content_type = "audio/mpeg",
        _ => content_type = "video/mp4"
    }
    HttpResponse::Ok().content_type(content_type).body(vec)
    // HttpResponse::Ok().body(form)
}


#[post("/api/manifest")]
async fn manifest(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> HttpResponse {
    let mut file_path_vec = Vec::new();
    let mut file_path = "".to_string();
    let mut file_name = "".to_string();

    for f in form.files {
        file_name = f.file_name.unwrap();
        file_path = format!("./original_read/{}", file_name.clone());
        // signed_file_path = format!("./signed/{}", file_name.clone());
        file_path_vec.push(file_path.clone());
        // file_path_vec.push(signed_file_path.clone());
        println!("{}", file_path.clone());
        log::info!("saving to {}", file_path.clone());
        f.file.persist(file_path.clone()).unwrap();
        
    }
    println!("Uploaded file");
    // let certificate = azure_func::main().await;
    //println!(certificate);
    // let certificate = azure_func::get_credentials().await.unwrap();
    // dbg!(&certificate);
    let manifest_report = c2pa_func::read_manifest(file_path).await.unwrap();
    let json_manifest_report = serde_json::to_string(&manifest_report).unwrap();
    HttpResponse::Ok().content_type("json/application").body(json_manifest_report)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("creating temporary upload directory");
    std::fs::create_dir_all("./original")?;
    std::fs::create_dir_all("./signed")?;
    std::fs::create_dir_all("./original_read")?;

    // let certificate = azure_func::main().await.unwrap();
    // dbg!(&certificate);
    // println!(certificate);
    // println!(format!("{}", string(&certificate)));
    log::info!("starting HTTP server");

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };
    HttpServer::new(|| {
        App::new()
            .service(file_and_manifest)
            .service(manifest)
    })
    .bind((Ipv4Addr::UNSPECIFIED, port))?
    .run()
    .await
}
