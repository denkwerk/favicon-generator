//! Turns settings and a source image into the output files, in memory.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufWriter, Cursor};

use anyhow::Result;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ImageBuffer, ImageEncoder, Pixel, PixelWithColorType, RgbaImage};

use crate::assets;
use crate::cli::Snippet;
use crate::config::Settings;
use crate::source::{self, Source};
use crate::spec;

/// A generated file, relative to the output directory.
#[derive(Debug)]
pub struct OutputFile {
    pub name: String,
    pub data: Vec<u8>,
}

/// Sizes of the transparent `favicon-{N}x{N}.png` files.
pub fn png_sizes(settings: &Settings) -> BTreeSet<u32> {
    let mut sizes = BTreeSet::from([spec::ICON_SIZE]);
    if settings.legacy {
        sizes.extend(spec::LEGACY_PNG_SIZES);
    }
    if settings.manifest.is_some() {
        sizes.extend(spec::MANIFEST_SIZES);
    }
    if settings.windows.is_some() {
        sizes.extend(spec::TILE_SIZES);
        sizes.insert(spec::TILE_IMAGE_SIZE);
    }
    sizes
}

fn ico_sizes(settings: &Settings) -> &'static [u32] {
    if settings.legacy {
        spec::LEGACY_ICO_SIZES
    } else {
        spec::ICO_SIZES
    }
}

/// Generates every file the settings ask for. Nothing is written to disk.
pub fn generate(settings: &Settings, source: &Source) -> Result<Vec<OutputFile>> {
    let png_sizes = png_sizes(settings);
    let apple_sizes: Vec<Option<u32>> = match &settings.apple_touch_icon {
        None => Vec::new(),
        Some(_) if settings.legacy => std::iter::once(None)
            .chain(spec::LEGACY_APPLE_TOUCH_SIZES.iter().copied().map(Some))
            .collect(),
        Some(_) => vec![None],
    };

    // Render each distinct size once.
    let sizes: BTreeSet<u32> = png_sizes
        .iter()
        .copied()
        .chain(ico_sizes(settings).iter().copied())
        .chain(
            apple_sizes
                .iter()
                .map(|size| size.unwrap_or(spec::APPLE_TOUCH_SIZE)),
        )
        .collect();
    let renders = sizes
        .into_iter()
        .map(|size| Ok((size, source.render(size)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;

    let mut files = vec![OutputFile {
        name: "favicon.ico".into(),
        data: encode_ico(&renders, ico_sizes(settings))?,
    }];
    if let Some(svg) = source.svg_bytes() {
        files.push(OutputFile {
            name: "favicon.svg".into(),
            data: svg.to_vec(),
        });
    }
    for &size in &png_sizes {
        files.push(OutputFile {
            name: spec::png_name(size),
            data: encode_png(&renders[&size])?,
        });
    }

    if let Some(apple) = &settings.apple_touch_icon {
        for size in apple_sizes {
            let render = &renders[&size.unwrap_or(spec::APPLE_TOUCH_SIZE)];
            let png = encode_png(&source::flatten(render, apple.background))?;
            if settings.legacy {
                files.push(OutputFile {
                    name: spec::apple_touch_name(size, true),
                    data: png.clone(),
                });
            }
            files.push(OutputFile {
                name: spec::apple_touch_name(size, false),
                data: png,
            });
        }
    }

    if let Some(manifest) = &settings.manifest
        && manifest.maskable
    {
        let background = manifest
            .background_color
            .as_deref()
            .map(|color| crate::cli::rgb(color).expect("validated color"))
            .unwrap_or([255, 255, 255]);
        for &size in spec::MANIFEST_SIZES {
            let inner = source.render((size as f32 * spec::MASKABLE_SCALE).round() as u32)?;
            files.push(OutputFile {
                name: spec::maskable_name(size),
                data: encode_png(&source::pad(&inner, size, background))?,
            });
        }
    }

    let cfg = assets::Config {
        settings,
        has_svg: source.svg_bytes().is_some(),
    };
    if let Some(manifest) = assets::manifest(&cfg) {
        files.push(OutputFile {
            name: "manifest.json".into(),
            data: manifest.into_bytes(),
        });
    }
    if let Some(browserconfig) = assets::browserconfig(&cfg) {
        files.push(OutputFile {
            name: "browserconfig.xml".into(),
            data: browserconfig.into_bytes(),
        });
    }
    for snippet in &settings.snippets {
        let (name, text) = match snippet {
            Snippet::Html => ("favicon.html", assets::html_snippet(&cfg)),
            Snippet::Json => ("favicon-head.json", assets::json_snippet(&cfg)),
            Snippet::Nuxt => ("nuxt-head.ts", assets::nuxt_snippet(&cfg)),
            Snippet::None => continue,
        };
        files.push(OutputFile {
            name: name.into(),
            data: text.into_bytes(),
        });
    }
    Ok(files)
}

fn encode_png<P>(image: &ImageBuffer<P, Vec<u8>>) -> Result<Vec<u8>>
where
    P: Pixel<Subpixel = u8> + PixelWithColorType,
{
    let mut out = Vec::new();
    PngEncoder::new_with_quality(
        Cursor::new(&mut out),
        CompressionType::Best,
        FilterType::Adaptive,
    )
    .write_image(image.as_raw(), image.width(), image.height(), P::COLOR_TYPE)?;
    Ok(out)
}

fn encode_ico(renders: &BTreeMap<u32, RgbaImage>, sizes: &[u32]) -> Result<Vec<u8>> {
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for &size in sizes {
        let frame = ico::IconImage::from_rgba_data(size, size, renders[&size].as_raw().clone());
        // 32-bit BMP frames for maximum compatibility; PNG for 256px keeps the file small.
        let entry = if size < 256 {
            ico::IconDirEntry::encode_as_bmp(&frame)?
        } else {
            ico::IconDirEntry::encode_as_png(&frame)?
        };
        dir.add_entry(entry);
    }
    let mut out = Vec::new();
    dir.write(BufWriter::new(&mut out))?;
    Ok(out)
}
