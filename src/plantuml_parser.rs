use crate::md2html::Html;
use crate::uml;
use lazy_static::lazy_static;
use regex::{Captures, Regex};

pub fn replace_plantuml_with_images(html: &Html) -> String {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"<pre><code>(@startuml (\w+)(.|\n)+?)</code></pre>").unwrap();
    }
    let mut idx = 0;
    RE.replace_all(html, |cap: &Captures| {
        let uml = cap[1].replace("&gt;", ">");
        let path = format!("/tmp/ripmd/{}.svg", idx);
        println!("uml {} {}", uml, path);
        uml::convert(&uml, &path).unwrap();
        idx += 1;
        format!(
            r#"
            <p>
              <a target="_blank" rel="noopener noreferrer" href="{}">
                <img src="{}" alt="{}" style="max-width: 100%;"> 
              </a>
            </p>
            "#,
            &path[1..],
            &path[1..],
            &path[1..]
        )
    })
    .into_owned()
}
