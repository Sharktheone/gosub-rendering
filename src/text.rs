use std::sync::Arc;

use gosub_styling::prerender_text::PrerenderText;
use rust_fontconfig::FcPattern;
use vello::glyph::Glyph;
use vello::kurbo::Affine;
use vello::peniko::{Blob, BrushRef, Font, StyleRef};
use vello::Scene;
use vello::skrifa::{FontRef, MetadataProvider};
use vello::skrifa::instance::Size;

use crate::FONT_CACHE;

pub struct TextRenderer {
    font: Font,
    font_size: f32,
    pub line_height: f32,
}

impl TextRenderer {
    pub fn new(font_family: Vec<String>, font_size: f32) -> Self {
        let cache = &*FONT_CACHE;

        let font_path = font_family.into_iter().find_map(|family| {
            cache.query(&FcPattern {
                family: Some(family),
                ..Default::default()
            })
        }).expect("No font found");

        let font_bytes = std::fs::read(&font_path.path).expect("Failed to read font file");
        let font = Font::new(Blob::new(Arc::new(font_bytes)), 0);

        let font_ref = to_font_ref(&font).expect("Failed to get font ref");

        let axes = font_ref.axes();
        let fs = Size::new(font_size);
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations.iter().copied());

        let metrics = font_ref.metrics(fs, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;

        Self { font, font_size, line_height }
    }

    pub fn new_with_font(font: Font, font_size: f32) -> Self {
        let font_ref = to_font_ref(&font).expect("Failed to get font ref");

        let axes = font_ref.axes();
        let fs = Size::new(font_size);
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations.iter().copied());

        let metrics = font_ref.metrics(fs, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;

        Self { font, font_size, line_height }
    }

    pub fn render_text<'a>(
        &self,
        text: &str,
        scene: &'a mut Scene,
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        style: impl Into<StyleRef<'a>>,
        glyph_transform: Option<Affine>,
    ) {
        self.render_custom_font_text(text, scene, brush, transform, glyph_transform, style, &self.font);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_custom_font_text<'a>(
        &self,
        text: &str,
        scene: &'a mut Scene,
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        font: &Font,
    ) {
        let font_ref = to_font_ref(font).expect("Failed to get font ref");

        let brush = brush.into();
        let axes = font_ref.axes();
        let font_size = Size::new(self.font_size);
        let char_map = font_ref.charmap();
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations.iter().copied());
        let glyph_metrics = font_ref.glyph_metrics(font_size, &var_loc);


        let mut pen_x = 0f32;

        scene
            .draw_glyphs(&self.font)
            .font_size(self.font_size)
            .transform(transform)
            .glyph_transform(glyph_transform)
            .normalized_coords(var_loc.coords())
            .brush(brush)
            .draw(style, text.chars().filter_map(|c| {
                if c == '\n' {
                    return None;
                }

                let gid = char_map.map(c).unwrap_or_default();
                let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                let x = pen_x;
                pen_x += advance;

                Some(Glyph {
                    id: gid.to_u16() as u32,
                    x,
                    y: 0.0,
                })
            }));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_multiline_custom_font_text<'a>(
        &self,
        text: &str,
        scene: &'a mut Scene,
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        font: &Font,
    ) {
        let font_ref = to_font_ref(font).expect("Failed to get font ref");

        let brush = brush.into();
        let char_map = font_ref.charmap();
        let axes = font_ref.axes();
        let font_size = Size::new(self.font_size);
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations.iter().copied());
        let glyph_metrics = font_ref.glyph_metrics(font_size, &var_loc);

        let metrics = font_ref.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;


        let mut pen_x = 0f32;
        let mut pen_y = 0f32;

        scene
            .draw_glyphs(&self.font)
            .font_size(self.font_size)
            .transform(transform)
            .glyph_transform(glyph_transform)
            .brush(brush)
            .draw(style, text.chars().filter_map(|c| {
                if c == '\n' {
                    pen_y += line_height;
                    pen_x = 0.0;
                    return None;
                }

                let gid = char_map.map(c).unwrap_or_default();
                let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                let x = pen_x;
                pen_x += advance;

                Some(Glyph {
                    id: gid.to_u16() as u32,
                    x,
                    y: pen_y,
                })
            }));
    }

    pub fn show_text<'a>(
        &self,
        prerendered: &PrerenderText,
        scene: &'a mut Scene,
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        style: impl Into<StyleRef<'a>>,
        glyph_transform: Option<Affine>,
    ) {
        let brush = brush.into();
        
        scene
            .draw_glyphs(&self.font)
            .font_size(self.font_size)
            .transform(transform)
            .glyph_transform(glyph_transform)
            .brush(brush)
            .draw(style, prerendered.glyphs.clone().into_iter())
    }
}


fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}