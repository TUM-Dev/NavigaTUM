use cached::lazy_static::lazy_static;
use image::Rgba;
use imageproc::definitions::HasBlack;
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};

lazy_static! {
    pub static ref CANTARELL_BOLD: Font<'static> =
        Font::try_from_bytes(include_bytes!("static/font/Cantarell-Bold.ttf")).unwrap();
    pub static ref CANTARELL_REGULAR: Font<'static> =
        Font::try_from_bytes(include_bytes!("static/font/Cantarell-Regular.ttf")).unwrap();
}
const SCALE: Scale = Scale { x: 35.0, y: 35.0 };

pub(crate) struct OverlayText {
    x: i32,
    y: i32,
    text: String,
    font: &'static Font<'static>,
}

impl OverlayText {
    pub fn with(text: &str, font: &'static Font<'static>) -> Self {
        Self {
            x: 0,
            y: 0,
            text: text.to_string(),
            font,
        }
    }
    /// x and y are in pixels from bottom right corner
    pub fn at(self, x: i32, y: i32) -> Self {
        Self { x, y, ..self }
    }
    pub fn draw_onto(self, img: &mut image::RgbaImage) {
        let (w, _) = text_size(SCALE, self.font, &self.text);
        draw_text_mut(
            img,
            Rgba::black(),
            img.width() as i32 - w - self.x,
            img.height() as i32 - self.y,
            SCALE,
            self.font,
            &self.text,
        );
    }
}
