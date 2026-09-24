//! Which files are generated and where they are referenced.

/// Frames in `favicon.ico`.
pub const ICO_SIZES: &[u32] = &[16, 32, 48];
/// Frames in `favicon.ico` with `legacy`.
pub const LEGACY_ICO_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];

/// The PNG favicon linked from `<head>`.
pub const ICON_SIZE: u32 = 96;
/// Every `favicon-{N}x{N}.png` written with `legacy`.
pub const LEGACY_PNG_SIZES: &[u32] = &[
    16, 32, 57, 60, 70, 72, 76, 96, 114, 120, 128, 144, 150, 152, 180, 192, 310, 384, 512,
];
/// `<link rel="icon" type="image/png">` sizes with `legacy`.
pub const LEGACY_ICON_LINK_SIZES: &[u32] = &[16, 32, 96, 192];

/// `apple-touch-icon.png`.
pub const APPLE_TOUCH_SIZE: u32 = 180;
/// Extra `apple-touch-icon-{N}x{N}.png` files with `legacy`; every Apple touch
/// icon then also gets a `-precomposed` copy.
pub const LEGACY_APPLE_TOUCH_SIZES: &[u32] = &[120, 152];
/// `<link rel="apple-touch-icon" sizes=…>` entries with `legacy`, pointing at `favicon-{N}x{N}.png`.
pub const LEGACY_APPLE_TOUCH_LINK_SIZES: &[u32] = &[57, 60, 72, 76, 114, 120, 144, 152, 180];

/// Icons listed in `manifest.json`, unless `manifest.iconSizes` is set.
pub const DEFAULT_MANIFEST_SIZES: &[u32] = &[192, 512];
/// Largest size `manifest.iconSizes` accepts.
pub const MAX_ICON_SIZE: u32 = 4096;
/// Share of a maskable icon covered by the image; the rest is background, so
/// that Android's masks do not cut into it.
pub const MASKABLE_SCALE: f32 = 0.6;

/// Square tiles in `browserconfig.xml`.
pub const TILE_SIZES: &[u32] = &[70, 150, 310];
/// `msapplication-TileImage`.
pub const TILE_IMAGE_SIZE: u32 = 144;

pub fn png_name(size: u32) -> String {
    format!("favicon-{size}x{size}.png")
}

pub fn maskable_name(size: u32) -> String {
    format!("favicon-maskable-{size}x{size}.png")
}

pub fn apple_touch_name(size: Option<u32>, precomposed: bool) -> String {
    let base = match size {
        Some(size) => format!("apple-touch-icon-{size}x{size}"),
        None => "apple-touch-icon".to_owned(),
    };
    let suffix = if precomposed { "-precomposed" } else { "" };
    format!("{base}{suffix}.png")
}
