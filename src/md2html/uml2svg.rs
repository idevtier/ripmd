use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
};

use super::Html;

pub type Result<T> = std::io::Result<T>;

/// Converts given UML text to svg by using
/// `plantuml` app. Returns `UML` if success
pub(crate) fn convert(uml: &str) -> Result<Html> {
    println!("Converting uml {} to svg", uml);
    let mut child = Command::new("plantuml")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-tsvg", uml, "-pipe"])
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or(std::io::ErrorKind::NotFound)?;
    let uml = uml.to_owned();
    thread::spawn(move || {
        stdin
            .write_all(uml.to_owned().as_bytes())
            .expect("Failed to write to stdin");
    });

    let uml = child.wait_with_output()?;
    Ok(String::from_utf8(uml.stdout).unwrap())
}
