use std::fs;

use anyhow::bail;
use gosub_html5::parser::document::Document;
use gosub_html5::parser::document::DocumentBuilder;
use gosub_html5::parser::Html5Parser;
use gosub_shared::bytes::{CharIterator, Confidence, Encoding};
use gosub_styling::render_tree::{generate_render_tree, RenderTree};
use url::Url;
use vello::Scene;
use gosub_rendering::WindowState;

fn main() -> anyhow::Result<()> {
    let args = clap::Command::new("Gosub Rendering Test")
        .version("0.1.0")
        .arg(
            clap::Arg::new("url")
                .help("The url to load")
                .required(true)
                .index(1),
        ).get_matches();

    let url = args.get_one::<String>("url").unwrap();
    let render_tree = load_html_rendertree(url)?;
    

    let mut render_scene = |scene: &mut Scene| {
        render_render_tree(scene, &render_tree)
    };

    let window = WindowState::new(&mut render_scene)?;

    window.start()?;

    Ok(())
}


fn load_html_rendertree(str_url: &str) -> anyhow::Result<RenderTree> {
    let url = Url::parse(str_url)?;
    let html = if url.scheme() == "http" || url.scheme() == "https" {
        // Fetch the html from the url
        let response = ureq::get(url.as_ref()).call()?;
        if response.status() != 200 {
            bail!(format!(
                "Could not get url. Status code {}",
                response.status()
            ));
        }
        response.into_string()?
    } else if url.scheme() == "file" {
        fs::read_to_string(str_url.trim_start_matches("file://"))?
    } else {
        bail!("Unsupported url scheme: {}", url.scheme());
    };

    let mut chars = CharIterator::new();
    chars.read_from_str(&html, Some(Encoding::UTF8));
    chars.set_confidence(Confidence::Certain);

    let doc_handle = DocumentBuilder::new_document(Some(url));
    let _parse_errors =
        Html5Parser::parse_document(&mut chars, Document::clone(&doc_handle), None)?;

    generate_render_tree(Document::clone(&doc_handle))
}


fn render_render_tree(_scene: &mut Scene, _render_tree: &RenderTree) {
    todo!("Render the render tree")
}