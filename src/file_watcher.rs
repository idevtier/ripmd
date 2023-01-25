use std::{
    fs::{self, File},
    io::Read,
    thread::{self, JoinHandle},
    time::Duration,
};

use sha1::{Digest, Sha1};

const CHUNK_SIZE: usize = 1024;

pub fn watch<F>(path: String, mut producer: F) -> JoinHandle<()>
where
    F: FnMut(String) + Send + 'static,
{
    thread::spawn(move || {
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
                        producer(data);
                    }
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
    })
}
