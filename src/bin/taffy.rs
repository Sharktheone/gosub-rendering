use std::fs;
use std::sync::Mutex;

use anyhow::bail;
use gosub_html5::node::NodeId;
use gosub_html5::parser::document::{Document, DocumentBuilder};
use gosub_html5::parser::Html5Parser;
use gosub_rendering::layout::generate_taffy_tree;
use gosub_shared::bytes::{CharIterator, Confidence, Encoding};
use gosub_styling::css_colors::RgbColor;
use gosub_styling::css_values::CssValue;
use gosub_styling::render_tree::{generate_render_tree, RenderNodeData, RenderTree};
use lazy_static::lazy_static;
use taffy::{AvailableSpace, NodeId as TaffyID, Size, TaffyTree, TraversePartialTree};
use url::Url;
use vello::kurbo::{Affine, Rect, RoundedRect};
use vello::peniko::{Color, Fill};
use vello::Scene;

use gosub_rendering_poc::image::ImageCache;
use gosub_rendering_poc::text::TextRenderer;
use gosub_rendering_poc::WindowState;

lazy_static! {
    static ref IMAGE_CACHE: Mutex<ImageCache> = Mutex::new(ImageCache::default());
}


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

    let mut render_tree = load_html_rendertree(url)?;

    let (mut taffy_tree, root) = generate_taffy_tree(&mut render_tree)?;

    taffy_tree.compute_layout(root, Size {
        width: AvailableSpace::Definite(1920.0),
        height: AvailableSpace::Definite(1080.0),
    }).expect("Failed to compute layout");


    taffy_tree.print_tree(root);

    // return Ok(());


    let last_size = (0, 0);

    let mut render_scene = |scene: &mut Scene, size: (usize, usize)| {
        if size != last_size {
            let size = Size {
                width: AvailableSpace::Definite(size.0 as f32),
                height: AvailableSpace::Definite(size.1 as f32),
            };
            taffy_tree.compute_layout(root, size).expect("Failed to compute layout");
        }
        render_render_tree(scene, size, &render_tree, &taffy_tree, root);
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


fn render_render_tree(scene: &mut Scene, size: (usize, usize), render_tree: &RenderTree, layout: &TaffyTree<NodeId>, root: TaffyID) {
    let bg = Rect::new(0.0, 0.0, size.0 as f64, size.1 as f64);
    scene.fill(Fill::NonZero, Affine::IDENTITY, Color::BLACK, None, &bg);

    render_with_children(root, render_tree, layout, scene);
}

fn render_with_children(id: TaffyID, render_tree: &RenderTree, layout: &TaffyTree<NodeId>, scene: &mut Scene) {
    let err = render_node(id, render_tree, layout, scene);
    if let Err(e) = err {
        eprintln!("Error rendering node: {:?}", e);
    }

    for child in layout.child_ids(id) {
        render_with_children(child, render_tree, layout, scene);
    }
}


fn render_node(id: TaffyID, render_tree: &RenderTree, layout: &TaffyTree<NodeId>, scene: &mut Scene) -> anyhow::Result<()> {
    let Some(gosub_id) = layout.get_node_context(id) else {
        return Err(anyhow::anyhow!("Node context not found"));
    };

    let gosub_id = *gosub_id;

    let node_layout = layout.layout(id)?;

    let pos_x = node_layout.location.x as f64;
    let pos_y = node_layout.location.y as f64;

    let node = render_tree.get_node(gosub_id).unwrap();
    if let RenderNodeData::Text(text) = &node.data {
        let text = &text.text;

        let ff;
        if let Some(prop) = render_tree.get_property(gosub_id, "font-family") {
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


        if let Some(mut prop) = render_tree.get_property(gosub_id, "font-size") {
            // prop.compute_value();

            fs = if let CssValue::String(fs) = prop.actual {
                if fs.ends_with("px") {
                    fs.trim_end_matches("px").parse::<f32>().unwrap_or(12.0)
                } else {
                    12.0
                }
            } else {
                12.0
            };
        } else {
            fs = 12.0
        };


        let renderer = TextRenderer::new(ff, fs);

        let color;

        if let Some(mut prop) = render_tree.get_property(gosub_id, "color") {
            // prop.compute_value();

            color = if let CssValue::String(color) = prop.actual {
                RgbColor::from(color.as_str())
            } else {
                RgbColor::new(255.0, 255.0, 255.0, 255.0)
            };
        } else {
            color = RgbColor::new(255.0, 255.0, 255.0, 255.0)
        };

        let color = Color::rgba8(color.r as u8, color.g as u8, color.b as u8, color.a as u8);

        let affine = Affine::translate((
            pos_x,
            pos_y,
        ));

        renderer.render_text(text, scene, color, affine, Fill::NonZero, None);
        return Ok(());
    }

    let mut color = RgbColor::new(0.0, 0.0, 0.0, 0.0);

    if let Some(mut prop) = render_tree
        .get_property(gosub_id, "background-color") {
        // prop.compute_value();
        if let CssValue::String(clr) = prop.actual {
            let clr = RgbColor::from(clr.as_str());
            color = clr;
        }
    }

    let mut border_radius = 0.0;

    if let Some(mut prop) = render_tree.get_property(gosub_id, "border-radius") {
        border_radius = prop.actual.unit_to_px() as f64;
    };

    let color = Color::rgba8(color.r as u8, color.g as u8, color.b as u8, color.a as u8);

    if let RenderNodeData::Element(e) = &node.data {
        if e.name == "img" {
            let Some(src) = e.attributes.get("src") else {
                return Err(anyhow::anyhow!("No src attribute found for img"));
            };


            let Ok(mut img_cache) = IMAGE_CACHE.try_lock() else {
                return Err(anyhow::anyhow!("Failed to lock image cache"));
            };

            let Ok(img) = img_cache.from_file(src) else {
                // return Err(anyhow::anyhow!("Failed to load image"));
                return Ok(());
            };

            scene.draw_image(&img,
                             Affine::translate((pos_x, pos_y)) * Affine::scale((node_layout.size.width / img.width as f32) as f64),
            );

            return Ok(());
        }
    }

    let x2 = node_layout.size.width as f64 + pos_x;
    let y2 = node_layout.size.height as f64 + pos_y;

    let rect = RoundedRect::new(pos_x, pos_y, x2, y2, border_radius);

    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);

    Ok(())
}


fn calculate_styles(render_tree: &mut RenderTree) {
    calculate_styles_for_node(NodeId::root(), render_tree);
}

fn calculate_styles_for_node(id: NodeId, render_tree: &mut RenderTree) {
    let Some(node) = render_tree.nodes.get_mut(&id) else {
        return;
    };

    node.properties.properties.iter_mut().for_each(|(_, prop)| {
        prop.compute_value();
    });

    for child in &node.children.clone() {
        calculate_styles_for_node(*child, render_tree);
    }
}
