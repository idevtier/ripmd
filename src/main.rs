mod file_watcher;
mod md2html;
mod server;

use clap::Parser;
use md2html::GithubMd2HtmlConverter;
use resolve_path::PathResolveExt;
use std::fs;
use std::sync::mpsc;
use std::thread;

pub enum WsUpdate {
    ReloadClient,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub github_token: String,

    #[arg(short, long, default_value_t = 8080)]
    pub fronted_port: u16,

    #[arg(short, long, default_value_t = 8089)]
    pub reloader_port: u16,
}

fn main() {
    let mut args = Args::parse();

    if args.path.starts_with('~') {
        args.path = resolve_path(&args.path);
    }

    let mut converter = GithubMd2HtmlConverter::new(&args.github_token);

    create_temp_dir_if_needed();

    let (ssender, sreceiver) = mpsc::channel();
    let (wsender, wreceiver) = mpsc::channel();

    let handle = file_watcher::watch(args.path.to_owned(), move |md| {
        let html = converter.convert(&md);
        match html {
            Err(e) => println!("Failed to load html: {:?}", e),
            Ok(html) => {
                // TODO: Refactor unwrap
                ssender.send(html).unwrap();
                wsender.send(WsUpdate::ReloadClient).unwrap();
            }
        }
    });

    let base_path = get_base_path(&args.path);
    let fronted_address = format!("localhost:{}", args.fronted_port);
    let reloader_address = format!("localhost:{}", args.reloader_port);

    thread::spawn(move || server::serve(&fronted_address, sreceiver, base_path));
    thread::spawn(move || server::ws(&reloader_address, wreceiver));

    handle
        .join()
        .expect("Failed to start markdown file watcher");
}

fn create_temp_dir_if_needed() {
    if let Err(e) = fs::create_dir("/tmp/ripmd") {
        println!("Tmp dir already exists: {}", e);
    }
}

fn get_base_path(path: &str) -> String {
    let splited: Vec<_> = path.split('/').collect();
    splited[..splited.len() - 1].join("/")
}

fn resolve_path(path: &str) -> String {
    path.try_resolve()
        .expect("Failed to resolve path")
        .to_str()
        .unwrap()
        .to_owned()
}
