use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{collections::HashMap, fs, io::Read};

use clap::Parser;
use tiny_http::{Response, Server, StatusCode};
use uuid::Uuid;
use walkdir::WalkDir;

use downloader_core::{Manifest, PatchFile, Provider};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Create a manifest instead of running the server
    #[arg(short, long, default_value_t = false)]
    create: bool,
    /// Directory to scan for files
    #[arg(short, long, default_value = "files", requires = "create")]
    dir: String,
    /// Output manifest file
    #[arg(short, long, default_value = "manifest.json", requires = "create")]
    output: String,
    /// Manifest version
    #[arg(
        short = 'm',
        long = "manifest-version",
        default_value = "1.0",
        requires = "create"
    )]
    manifest_version: String,
    /// Base URL for file links
    #[arg(
        short,
        long,
        default_value = "http://localhost:8080/",
        requires = "create"
    )]
    base_url: String,
    /// Throttle in ms per chunk (0 = no throttle)
    #[arg(short, long, default_value_t = 0, conflicts_with = "create")]
    throttle: u64,
    /// Directory to serve files from (for server mode)
    #[arg(long, default_value = "files", conflicts_with = "create")]
    serve_dir: String,
}

fn create_manifest(config: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(&config.dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let rel_path = path
            .strip_prefix(&config.dir)
            .unwrap()
            .to_string_lossy()
            .replace("\\", "/");
        let rel_path = rel_path
            .trim_start_matches('/')
            .trim_start_matches('\\')
            .to_string();
        let manifest_path = format!("files/{rel_path}");

        let mut file = fs::File::open(path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let hash = format!("{:x}", md5::compute(&buf));
        let size = buf.len() as i64;

        let url = format!("{}/{}", config.base_url.trim_end_matches('/'), rel_path);
        let mut urls = HashMap::new();
        for provider in [Provider::Cloudflare, Provider::DigitalOcean, Provider::None] {
            urls.insert(provider, url.clone());
        }

        files.push(PatchFile {
            path: manifest_path,
            hash,
            size,
            custom: true,
            urls: urls.clone(),
        });
    }

    let manifest = Manifest {
        version: config.manifest_version.clone(),
        uuid: Uuid::new_v4().to_string(),
        files,
        removals: None,
    };

    let json = serde_json::to_string_pretty(&manifest).unwrap();
    fs::write(&config.output, json).unwrap();
    println!("Manifest written to {}", config.output);
    Ok(())
}

fn run_server(serve_dir: String, throttle: u64) {
    let server = Server::http("0.0.0.0:8080").unwrap();
    println!("Starting local server on http://localhost:8080/");

    for request in server.incoming_requests() {
        let url = request.url();
        if url.starts_with("/files/") {
            // Serve from files directory
            let rel_path = url.strip_prefix("/files/").unwrap_or("");
            let mut file_path = PathBuf::from(&serve_dir);
            file_path.push(rel_path);

            match fs::File::open(&file_path) {
                Ok(mut file) => {
                    let mut buf = [0u8; 1024];
                    let mut response_data = Vec::new();
                    loop {
                        match file.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                response_data.extend_from_slice(&buf[..n]);
                                if throttle > 0 {
                                    thread::sleep(Duration::from_millis(throttle));
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    let response = Response::from_data(response_data);
                    request.respond(response).ok();
                }
                Err(_) => {
                    request.respond(Response::empty(StatusCode(404))).ok();
                }
            }
        } else {
            // Serve from current directory (e.g. manifest.json)
            let path = url.trim_start_matches('/');
            match fs::File::open(path) {
                Ok(mut file) => {
                    let mut body = Vec::new();
                    file.read_to_end(&mut body).ok();
                    let response = Response::from_data(body);
                    request.respond(response).ok();
                }
                Err(_) => {
                    request.respond(Response::empty(StatusCode(404))).ok();
                }
            }
        }
    }
}

fn main() {
    let config = Cli::parse();

    if config.create {
        create_manifest(&config).unwrap_or_else(|e| {
            eprintln!("Error creating manifest: {e}");
            std::process::exit(1);
        });
        return;
    }

    run_server(config.serve_dir, config.throttle);
}
