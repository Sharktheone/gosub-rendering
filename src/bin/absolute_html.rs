use std::fs;

use anyhow::bail;
use gosub_html5::parser::document::Document;
use gosub_html5::parser::document::DocumentBuilder;
use gosub_html5::parser::Html5Parser;
use gosub_rendering::WindowState;
use gosub_shared::bytes::{CharIterator, Confidence, Encoding};
use gosub_styling::css_colors::RgbColor;
use gosub_styling::css_values::CssValue;
use gosub_styling::render_tree::{generate_render_tree, RenderTree};
use url::Url;
use vello::kurbo::{Affine, Rect};
use vello::kurbo::RoundedRect;
use vello::peniko::Color;
use vello::peniko::Fill;
use vello::Scene;

fn main() -> anyhow::Result<()> {
    let args = clap::Command::new("Gosub Rendering Test")
        .version("0.1.0")
        .arg(
            clap::Arg::new("url")
                .help("The url to load")
                .required(true)
                .index(1),
        )
        .get_matches();

    let url = args.get_one::<String>("url").unwrap();
    let render_tree = load_html_rendertree(url)?;

    let mut render_scene = |scene: &mut Scene, size: (usize, usize)| render_render_tree(scene, size, &render_tree);

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

fn render_render_tree(scene: &mut Scene, size: (usize, usize), render_tree: &RenderTree) {
    
    let bg = Rect::new(0.0, 0.0, size.0 as f64, size.1 as f64);
    scene.fill(Fill::NonZero, Affine::IDENTITY, Color::BLACK, None, &bg);
    
    
    
    for (id, _node) in render_tree.nodes.iter() {
        println!("Node: {:?}", id);
        let Some(prop) = render_tree.get_property(*id, "position") else {
            continue;
        };

        let CssValue::String(pos) = prop.actual else {
            continue;
        };

        if pos != "absolute" {
            continue;
        }
        
        println!("...is absolute!");

        let mut top = f32::MIN;
        let mut left = f32::MIN;
        let mut right = f32::MIN;
        let mut bottom = f32::MIN;

        if let Some(prop) = render_tree.get_property(*id, "top") {
            if let CssValue::Unit(top_val, unit) = prop.actual {
                if unit == "px" {
                    top = top_val;
                }
            };
            
            
        };

        if let Some(prop) = render_tree.get_property(*id, "left") {
            if let CssValue::Unit(left_val, unit) = prop.actual {
                if unit == "px" {
                    left = left_val;
                }
            };

        };

        if let Some(prop) = render_tree.get_property(*id, "right") {
            if let CssValue::Unit(right_val, unit) = prop.actual {
                if unit == "px" {
                    right = right_val;
                }
            };
        };

        if let Some(prop) = render_tree.get_property(*id, "bottom") {
            if let CssValue::Unit(bottom_val, unit) = prop.actual {
                if unit != "px" {
                    bottom = bottom_val;
                }
            };

        };

        if top == f32::MIN && bottom != f32::MIN {
            top = size.1 as f32 - bottom;
        }

        if left == f32::MIN && right != f32::MIN {
            left = size.0 as f32 - right;
        }

        if top == f32::MIN || left == f32::MIN {
            continue;
        }

        let mut width = 0.0;
        let mut height = 0.0;

        if let Some(prop) = render_tree.get_property(*id, "width") {
            if let CssValue::Unit(width_val, unit) = prop.actual {
                if unit != "px" {
                    width = width_val;
                }
            };
        };

        if let Some(prop) = render_tree.get_property(*id, "height") {
            if let CssValue::Unit(height_val, unit) = prop.actual {
                if unit != "px" {
                    height = height_val;
                }
            };
        };

        let mut color = RgbColor::new(0, 0, 0, 0);

        if let Some(prop) = render_tree
            .get_property(*id, "background-color") {
            if let CssValue::Color(clr) = prop.actual {
                color = clr;
            }
        }

        let mut border_radius = 0.0;

        if let Some(prop) = render_tree.get_property(*id, "border-radius") {
            if let CssValue::Unit(val, unit) = prop.actual {
                if unit == "px" {
                    border_radius = val;
                }
            }
        }

        let color = Color::rgba8(color.r, color.g, color.b, color.a);
        if width == 0.0 || height == 0.0 {
            continue;
        }

        let x1 = left as f64;
        let y1 = top as f64;
        let x2 = (left + width) as f64;
        let y2 = (top + height) as f64;

        let rect = RoundedRect::new(x1, y1, x2, y2, border_radius as f64);

        scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);
    }
}


