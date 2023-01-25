use std::{collections::HashMap, fs, sync::Arc, thread};

use super::Html;
use lazy_static::lazy_static;
use regex::{Captures, Regex};

pub type Result<T> = std::io::Result<T>;

pub struct Plantuml {
    cache: HashMap<String, String>,
}

impl Plantuml {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn replace_plantuml_with_images<F>(&mut self, html: Html, converter: F) -> Result<Html>
    where
        F: Fn(&str) -> std::io::Result<String>,
        F: Send + Sync + 'static,
    {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"<pre><code>(@startuml ?(\w+)?(.|\n)+?)</code></pre>").unwrap();
        }

        let mut idx = 0;
        let mut handles = Vec::new();
        let converter = Arc::new(converter);

        let html = RE
            .replace_all(&html, |cap: &Captures| {
                let uml = cap[1].replace("&gt;", ">").replace("&lt;", "<");

                if let Some(name) = self.cache.get(&uml) {
                    let path = format!("/tmp/ripmd/{}.svg", name);
                    self.create_img_html(&path[1..], name)
                } else {
                    let name = if let Some(name) = cap.get(2) {
                        name.as_str().to_string()
                    } else {
                        idx += 1;
                        idx.to_string()
                    };
                    let path = format!("/tmp/ripmd/{}.svg", name);
                    let img_html = self.create_img_html(&path[1..], &name);
                    self.cache.insert(uml.clone(), name);

                    let converter = Arc::clone(&converter);
                    let handle = thread::spawn(move || {
                        let svg = converter(&uml);
                        (path, svg)
                    });
                    handles.push(handle);
                    img_html
                }
            })
            .into_owned();

        for handle in handles {
            match handle.join() {
                Ok((path, uml)) => fs::write(path, uml?)?,
                Err(e) => println!("Failed to convert plantuml to svg: {:?}", e),
            }
        }

        Ok(html)
    }

    fn create_img_html(&self, path: &str, alt: &str) -> String {
        format!(
            r#"
<p>
<a target="_blank" rel="noopener noreferrer" href="{}">
<img src="{}" alt="{}" style="max-width: 100%;"> 
</a>
</p>
            "#,
            path, path, alt
        )
    }
}
