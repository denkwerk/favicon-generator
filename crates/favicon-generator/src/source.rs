use std::path::Path;

use anyhow::{Context, Result, bail};
use image::{Rgb, RgbImage, RgbaImage, imageops::FilterType};
use resvg::{tiny_skia, usvg};

/// The image every icon is rendered from.
pub enum Source {
    /// Vector input: rendered natively at every target size for crisp results.
    Svg { tree: Box<usvg::Tree>, raw: Vec<u8> },
    /// Raster input: downscaled with a Lanczos filter.
    Raster(RgbaImage),
}

impl Source {
    pub fn load(path: &Path) -> Result<Self> {
        let raw =
            std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let is_svg = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("svg"));
        Self::decode(raw, is_svg, path)
    }

    /// Decodes the contents of the file at `path`.
    pub fn decode(raw: Vec<u8>, is_svg: bool, path: &Path) -> Result<Self> {
        if is_svg {
            Self::from_svg(raw, path.parent()).with_context(|| path.display().to_string())
        } else {
            let image = image::load_from_memory(&raw)
                .with_context(|| format!("failed to decode image {}", path.display()))?
                .into_rgba8();
            Ok(Source::Raster(image))
        }
    }

    /// Parses SVG data; relative `<image>` references resolve against `resources_dir`.
    pub fn from_svg(raw: Vec<u8>, resources_dir: Option<&Path>) -> Result<Self> {
        let mut options = usvg::Options {
            resources_dir: resources_dir.map(Path::to_path_buf),
            ..Default::default()
        };
        options.fontdb_mut().load_system_fonts();
        let tree = usvg::Tree::from_data(&raw, &options).context("failed to parse SVG")?;
        Ok(Source::Svg {
            tree: Box::new(tree),
            raw,
        })
    }

    /// Native size of the source, used to warn about upscaling.
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            Source::Svg { tree, .. } => (tree.size().width(), tree.size().height()),
            Source::Raster(image) => (image.width() as f32, image.height() as f32),
        }
    }

    pub fn svg_bytes(&self) -> Option<&[u8]> {
        match self {
            Source::Svg { raw, .. } => Some(raw),
            Source::Raster(_) => None,
        }
    }

    /// Renders a `size`×`size` icon. Non-square sources are fitted and centered
    /// on a transparent canvas.
    pub fn render(&self, size: u32) -> Result<RgbaImage> {
        match self {
            Source::Svg { tree, .. } => render_svg(tree, size),
            Source::Raster(image) => Ok(render_raster(image, size)),
        }
    }
}

fn render_svg(tree: &usvg::Tree, size: u32) -> Result<RgbaImage> {
    let Some(mut pixmap) = tiny_skia::Pixmap::new(size, size) else {
        bail!("invalid icon size {size}");
    };
    let (w, h) = (tree.size().width(), tree.size().height());
    let scale = size as f32 / w.max(h);
    let dx = (size as f32 - w * scale) / 2.0;
    let dy = (size as f32 - h * scale) / 2.0;
    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(dx, dy);
    resvg::render(tree, transform, &mut pixmap.as_mut());

    // tiny-skia stores premultiplied alpha; PNG/ICO expect straight alpha.
    let data = pixmap.take_demultiplied();
    Ok(RgbaImage::from_raw(size, size, data).expect("pixmap buffer matches its dimensions"))
}

fn render_raster(image: &RgbaImage, size: u32) -> RgbaImage {
    let (w, h) = image.dimensions();
    let scale = size as f32 / w.max(h) as f32;
    let nw = ((w as f32 * scale).round() as u32).clamp(1, size);
    let nh = ((h as f32 * scale).round() as u32).clamp(1, size);
    let resized = image::imageops::resize(image, nw, nh, FilterType::Lanczos3);

    let mut canvas = RgbaImage::new(size, size);
    image::imageops::overlay(
        &mut canvas,
        &resized,
        ((size - nw) / 2).into(),
        ((size - nh) / 2).into(),
    );
    canvas
}

/// Composites `image` onto an opaque background (iOS renders transparency as black).
pub fn flatten(image: &RgbaImage, background: [u8; 3]) -> RgbImage {
    RgbImage::from_fn(image.width(), image.height(), |x, y| {
        let pixel = image.get_pixel(x, y);
        let a = pixel[3] as u32;
        Rgb(std::array::from_fn(|c| {
            ((pixel[c] as u32 * a + background[c] as u32 * (255 - a) + 127) / 255) as u8
        }))
    })
}

/// Places `image` in the middle of an opaque `size`×`size` square, for maskable
/// icons whose edges Android may cut away.
pub fn pad(image: &RgbaImage, size: u32, background: [u8; 3]) -> RgbImage {
    let mut canvas = RgbaImage::from_pixel(
        size,
        size,
        image::Rgba([background[0], background[1], background[2], 255]),
    );
    let offset = ((size - image.width().min(size)) / 2).into();
    image::imageops::overlay(&mut canvas, image, offset, offset);
    flatten(&canvas, background)
}
