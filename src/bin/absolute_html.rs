use std::fs;

use anyhow::bail;
use gosub_html5::node::{NodeData, NodeId};
use gosub_html5::parser::document::Document;
use gosub_html5::parser::document::DocumentBuilder;
use gosub_html5::parser::Html5Parser;
use gosub_shared::bytes::{CharIterator, Confidence, Encoding};
use gosub_styling::css_colors::RgbColor;
use gosub_styling::css_values::CssValue;
use gosub_styling::render_tree::{generate_render_tree, RenderTree, RenderTreeNode};
use url::Url;
use vello::kurbo::{Affine, Rect, Stroke};
use vello::kurbo::RoundedRect;
use vello::peniko::Color;
use vello::peniko::Fill;
use vello::Scene;

use gosub_rendering::text::TextRenderer;
use gosub_rendering::WindowState;

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

    let Some(parent) = render_tree.nodes.get(&NodeId::root()) else {
        println!("no parent found");
        return
    };

    for child in &parent.children {
        render_with_children(*child, render_tree, scene, size);
    }
}

fn render_with_children(id: NodeId, render_tree: &RenderTree, scene: &mut Scene, size: (usize, usize)) {
    let Some(node) = render_tree.nodes.get(&id) else {
        return;
    };
    render_node(id, node, render_tree, scene, size);

    for child in &node.children {
        render_with_children(*child, render_tree, scene, size);
    }
}



fn render_node(id: NodeId, node: &RenderTreeNode, render_tree: &RenderTree, scene: &mut Scene, size: (usize, usize)) {
    if let NodeData::Text(text) = &node.data {
        let text = &text.value;

        let ff;
        if let Some(prop) = render_tree.get_property(id, "font-family") {
            ff = if let CssValue::String(font_family) = prop.actual {
                font_family
            } else {
                String::from("Arial")
            };
        } else {
            ff = String::from("Arial")
        };

        let ff = ff.trim().split(',').map(|ff| ff.to_string()).collect::<Vec<String>>();


        let fs;


        if let Some(mut prop) = render_tree.get_property(id, "font-size") {
            prop.compute_value();

            fs = if let CssValue::String(fs) = prop.actual {
                fs.parse::<f32>().unwrap_or(12.0)
            } else {
                12.0
            };
        } else {
            fs = 12.0
        };


        let renderer = TextRenderer::new(ff, fs);

        let color;

        if let Some(mut prop) = render_tree.get_property(id, "color") {
            prop.compute_value();

            color = if let CssValue::String(color) = prop.actual {
                RgbColor::from(color.as_str())
            } else {
                RgbColor::new(0, 0, 0, 255)
            };
        } else {
            color = RgbColor::new(0, 0, 0, 255)
        };

        let color = Color::rgba8(color.r, color.g, color.b, color.a);


        renderer.render_text(text, scene, color, Affine::IDENTITY, &Stroke::new(1.0), None);
        return;
    }

    let Some(mut prop) = render_tree.get_property(id, "position") else {
        return;
    };

    prop.compute_value();

    let CssValue::String(pos) = prop.actual else {
        return;
    };

    if pos != "absolute" {
        return;
    }

    let mut top = f64::MIN;
    let mut left = f64::MIN;
    let mut right = f64::MIN;
    let mut bottom = f64::MIN;

    if let Some(mut prop) = render_tree.get_property(id, "top") {
        prop.compute_value();
        if let CssValue::String(val) = prop.actual {
            if val.ends_with("px") {
                top = val.trim_end_matches("px").parse().unwrap();
            }
        };
    };

    if let Some(mut prop) = render_tree.get_property(id, "left") {
        prop.compute_value();
        if let CssValue::String(val) = prop.actual {
            if val.ends_with("px") {
                left = val.trim_end_matches("px").parse().unwrap();
            }
        };
    };

    if let Some(mut prop) = render_tree.get_property(id, "right") {
        prop.compute_value();
        if let CssValue::String(val) = prop.actual {
            if val.ends_with("px") {
                right = val.trim_end_matches("px").parse().unwrap();
            }
        };
    };

    if let Some(mut prop) = render_tree.get_property(id, "bottom") {
        prop.compute_value();
        if let CssValue::String(val) = prop.actual {
            if val.ends_with("px") {
                bottom = val.trim_end_matches("px").parse().unwrap();
            }
        };
    };

    if top == f64::MIN && bottom != f64::MIN {
        top = size.1 as f64 - bottom;
    }

    if left == f64::MIN && right != f64::MIN {
        left = size.0 as f64 - right;
    }

    if top == f64::MIN || left == f64::MIN {
        return;
    }

    let mut width = f64::MIN;
    let mut height = f64::MIN;

    if let Some(mut prop) = render_tree.get_property(id, "width") {
        prop.compute_value();
        if let CssValue::String(val) = prop.actual {
            if val.ends_with("px") {
                width = val.trim_end_matches("px").parse().unwrap();
            }
        };
    };

    if let Some(mut prop) = render_tree.get_property(id, "height") {
        prop.compute_value();
        if let CssValue::String(val) = prop.actual {
            if val.ends_with("px") {
                height = val.trim_end_matches("px").parse().unwrap();
            }
        };
    };

    let mut color = RgbColor::new(0, 0, 0, 0);

    if let Some(mut prop) = render_tree
        .get_property(id, "background-color") {
        prop.compute_value();
        if let CssValue::String(clr) = prop.actual {
            let clr = RgbColor::from(clr.as_str());
            color = clr;
        }
    }

    let mut border_radius = (0.0, 0.0, 0.0, 0.0);

    if let Some(mut prop) = render_tree.get_property(id, "border-radius") {
        prop.compute_value();
        if let CssValue::String(val) = prop.actual {
            let val = val.split(' ');
            let mut vals = val.map(|v| {
                if v.ends_with("px") {
                    v.trim_end_matches("px").parse::<f64>().unwrap()
                } else {
                    0.0
                }
            });
            let top_left = vals.next().unwrap_or(0.0);
            let top_right = vals.next().unwrap_or(top_left);
            let bottom_right = vals.next().unwrap_or(top_left);
            let bottom_left = vals.next().unwrap_or(top_right);

            border_radius = (top_left, top_right, bottom_right, bottom_left);
        };
    }

    let color = Color::rgba8(color.r, color.g, color.b, color.a);
    if width == 0.0 || height == 0.0 {
        return;
    }

    let x1 = left;
    let y1 = top;

    let x2 = if width == f64::MIN {
        size.0 as f64 - right
    } else {
        left + width
    };

    let y2 = if height == f64::MIN {
        size.1 as f64 - bottom
    } else {
        top + height
    };

    let rect = RoundedRect::new(x1, y1, x2, y2, border_radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);
}

