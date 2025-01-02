use ab_glyph::{FontArc, PxScale};
use image::Rgba;
use imageproc::definitions::HasBlack;
use imageproc::drawing::{draw_text_mut, text_size};
use std::fmt;
use std::fmt::Formatter;
use std::sync::OnceLock;

pub fn cantarell_bold() -> &'static FontArc {
    static CANTARELL_BOLD: OnceLock<FontArc> = OnceLock::new();
    CANTARELL_BOLD
        .get_or_init(|| FontArc::try_from_slice(include_bytes!("font/Cantarell-Bold.ttf")).unwrap())
}
pub fn cantarell_regular() -> &'static FontArc {
    static CANTARELL_REGULAR: OnceLock<FontArc> = OnceLock::new();
    CANTARELL_REGULAR.get_or_init(|| {
        FontArc::try_from_slice(include_bytes!("font/Cantarell-Regular.ttf")).unwrap()
    })
}
const SCALE: PxScale = PxScale { x: 35.0, y: 35.0 };

pub struct OverlayText {
    x: i32,
    y: i32,
    text: String,
    font: &'static FontArc,
}

impl fmt::Debug for OverlayText {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverlayText")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("text", &self.text)
            .finish()
    }
}

impl OverlayText {
    pub fn with(text: &str, font: &'static FontArc) -> Self {
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

    #[tracing::instrument(skip(img))]
    pub fn draw_onto(self, img: &mut image::RgbaImage) {
        let (w, _) = text_size(SCALE, self.font, &self.text);
        draw_text_mut(
            img,
            Rgba::black(),
            img.width() as i32 - w as i32 - self.x,
            img.height() as i32 - self.y,
            SCALE,
            self.font,
            &self.text,
        );
    }
}
