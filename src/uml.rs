use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    thread,
};

pub type Result<T> = std::io::Result<T>;

/// Converts given UML text to svg by using
/// `plantuml` app. Saves result in path from
/// `path` param.
pub fn convert(uml: &str, path: &str) -> Result<()> {
    println!("Got uml: {}", uml);
    let mut child = Command::new("plantuml")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-tsvg", &uml.to_owned(), "-pipe"])
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or(std::io::ErrorKind::NotFound)?;
    let uml = uml.to_owned();
    thread::spawn(move || {
        stdin
            .write_all(uml.to_owned().as_bytes())
            .expect("Failed to write to stdin");
    });

    child.wait()?;
    let uml = child.wait_with_output()?;
    fs::write(path, uml.stdout)?;
    Ok(())
}
