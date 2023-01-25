use std::collections::HashMap;

use reqwest::{
    self,
    blocking::ClientBuilder,
    header::{self, HeaderMap, HeaderValue},
};

use self::plantuml::Plantuml;

mod plantuml;
mod uml2svg;

pub type Html = String;
pub type Result<T> = core::result::Result<T, reqwest::Error>;

const ENDPOINT: &str = "https://api.github.com/markdown";

pub struct GithubMd2HtmlConverter {
    token: String,
    plantuml: Plantuml,
}

impl GithubMd2HtmlConverter {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            plantuml: Plantuml::new(),
        }
    }

    pub fn convert(&mut self, markdown: &str) -> Result<Html> {
        let client_builder = ClientBuilder::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(header::USER_AGENT, HeaderValue::from_static("hyper"));
        let mut request = HashMap::new();
        request.insert("text", markdown.to_owned());
        let client = client_builder.default_headers(headers).build()?;
        let res = client
            .post(ENDPOINT)
            .bearer_auth(&self.token)
            .json(&request)
            .send()?;

        let html = self
            .plantuml
            .replace_plantuml_with_images(res.text()?, uml2svg::convert)
            .unwrap();

        Ok(html)
    }
}
