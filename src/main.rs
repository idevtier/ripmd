mod md2html;
mod plantuml_parser;
mod server;
mod uml;

use clap::Parser;
use md2html::Html;
use md2html::Md2HtmlConverter;
use resolve_path::PathResolveExt;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Read;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

const CHUNK_SIZE: usize = 1024;

enum Message {
    FileUpdated(Html),
}

pub enum WsUpdate {
    ReloadClient,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub github_token: String,
}

fn main() {
    let mut args = Args::parse();

    if args.path.starts_with("~") {
        args.path = args.path.resolve().to_str().unwrap().to_owned();
    }

    if let Err(e) = fs::create_dir("/tmp/ripmd") {
        println!("Failed to create tmp dir {}", e);
    }

    let converter = Md2HtmlConverter::new(&args.github_token);

    let path = &args.path;
    let receiver = foo(path.to_owned(), move |sender, text| {
        let html = converter.convert(&text);
        match html {
            Err(e) => println!("Failed to load html: {:?}", e),
            Ok(html) => {
                sender.send(Message::FileUpdated(html)).unwrap();
            }
        }
    });

    let (ssender, sreceiver) = mpsc::channel();
    let (wsender, wreceiver) = mpsc::channel();
    let base_path = get_base_path(path);
    thread::spawn(move || server::serve("localhost:8080", sreceiver, base_path));
    thread::spawn(|| server::ws("localhost:8089", wreceiver));

    loop {
        if let Ok(message) = receiver.recv() {
            match message {
                Message::FileUpdated(mut html) => {
                    println!("File updated");
                    html = plantuml_parser::replace_plantuml_with_images(&html);
                    ssender.send(html).unwrap();
                    wsender.send(WsUpdate::ReloadClient).unwrap();
                }
            }
        }
    }
}

fn foo<F>(path: String, producer: F) -> Receiver<Message>
where
    F: Fn(&Sender<Message>, String) + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let _ = thread::spawn(move || {
        let mut last_hash = vec![0u8; 20];
        loop {
            match fs::metadata(&path) {
                Err(e) => println!("Got error: {:?}", e),
                Ok(_) => {
                    let mut hasher = Sha1::new();
                    let mut buffer = vec![0u8; CHUNK_SIZE];
                    let mut file = File::open(&path).unwrap();
                    loop {
                        let count = file.read(&mut buffer).unwrap();
                        hasher.update(&buffer);
                        if count == 0 {
                            break;
                        }
                    }
                    let result = hasher.finalize().into_iter().collect();
                    if last_hash != result {
                        last_hash = result;
                        let data = fs::read_to_string(&path).unwrap();
                        producer(&sender, data);
                    }
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
    });
    receiver
}

fn get_base_path(path: &str) -> String {
    let splited: Vec<_> = path.split("/").collect();
    splited[..splited.len() - 1].join("/")
}
