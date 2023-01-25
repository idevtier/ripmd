use crate::{md2html::Html, WsUpdate};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::{self, File},
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc::Receiver,
};
use tungstenite::{accept, Message};

static mut LAST_CONTENT: Option<Html> = None;

pub fn serve(address: &str, receiver: Receiver<Html>, base_path: String) {
    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &receiver, &base_path);
    }
}

pub fn ws(address: &str, receiver: Receiver<WsUpdate>) {
    let listener = TcpListener::bind(address).unwrap();
    for stream in listener.incoming() {
        let mut websocket = accept(stream.unwrap()).unwrap();
        if let Ok(msg) = receiver.recv() {
            match msg {
                WsUpdate::ReloadClient => {
                    websocket
                        .write_message(Message::Text("reload".to_owned()))
                        .unwrap();
                }
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, receiver: &Receiver<Html>, base_path: &str) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    if request_line == "GET / HTTP/1.1" {
        let status_line = "HTTP/1.1 200 OK";
        let mut contents = fs::read_to_string("static/template.html").unwrap();
        if let Ok(html) = receiver.try_recv() {
            contents = contents.replace("{{body}}", &html);
            update_last_content(&contents);
        } else {
            contents = get_last_content();
        }
        let length = contents.len();

        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).unwrap();
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"GET /(.+) HTTP/1.1").unwrap();
    }

    if let Some(image) = RE.captures(&request_line) {
        let (path, content_type) = if image[1].starts_with("tmp/ripmd") {
            let path = "/".to_owned() + &image[1];
            (path, "image/svg+xml")
        } else {
            let path = base_path.to_owned() + "/" + &image[1];
            (path, "image/png")
        };
        let img = File::open(path);
        if let Ok(mut img) = img {
            let mut buf = Vec::new();
            img.read_to_end(&mut buf).unwrap();

            let headers = [
                "HTTP/1.1 200 OK",
                &format!("Content-type: {}", content_type),
                "\r\n",
            ];
            let mut resp = headers.join("\r\n").into_bytes();
            resp.extend(buf);
            stream.write_all(&resp).unwrap();
        }
    }
}

fn update_last_content(new_contnet: &Html) {
    unsafe { LAST_CONTENT = Some(new_contnet.clone()) }
}

fn get_last_content() -> String {
    unsafe { LAST_CONTENT.clone().unwrap_or_else(|| "".to_owned()) }
}
