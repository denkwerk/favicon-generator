//! Which files are generated and where they are referenced.

/// Every `favicon-{N}x{N}.png` that is written.
pub const PNG_SIZES: [u32; 19] = [
    16, 32, 57, 60, 70, 72, 76, 96, 114, 120, 128, 144, 150, 152, 180, 192, 310, 384, 512,
];

/// Frames embedded in `favicon.ico`.
pub const ICO_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

/// Opaque `apple-touch-icon*.png` files: `None` is the unsuffixed 180×180 default.
pub const APPLE_TOUCH_FILES: [(Option<u32>, u32); 3] =
    [(None, 180), (Some(120), 120), (Some(152), 152)];

/// `<link rel="apple-touch-icon" sizes=…>` entries.
pub const APPLE_TOUCH_LINK_SIZES: [u32; 9] = [57, 60, 72, 76, 114, 120, 144, 152, 180];

/// `<link rel="icon" type="image/png" sizes=…>` entries.
pub const ICON_LINK_SIZES: [u32; 4] = [16, 32, 96, 192];

/// Icons listed in `manifest.json`.
pub const MANIFEST_SIZES: [u32; 8] = [72, 96, 128, 144, 152, 192, 384, 512];

/// Square tiles in `browserconfig.xml`.
pub const TILE_SIZES: [u32; 3] = [70, 150, 310];

/// `msapplication-TileImage`.
pub const TILE_IMAGE_SIZE: u32 = 144;

pub fn png_name(size: u32) -> String {
    format!("favicon-{size}x{size}.png")
}

pub fn apple_touch_names(suffix: Option<u32>) -> [String; 2] {
    let base = match suffix {
        Some(size) => format!("apple-touch-icon-{size}x{size}"),
        None => "apple-touch-icon".to_owned(),
    };
    [format!("{base}.png"), format!("{base}-precomposed.png")]
}
