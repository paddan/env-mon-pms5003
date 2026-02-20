use embedded_graphics::{prelude::*, primitives::Rectangle};
use u8g2_fonts::{
    fonts,
    types::{FontColor, VerticalPosition},
    FontRenderer,
};

use super::{
    DisplayColor, FontToken, ResolvedFont, U8g2FontToken, GAUGE_CENTER, STATUS_TEXT_CLEAR_PAD_BOTTOM,
    STATUS_TEXT_CLEAR_PAD_TOP, STATUS_TEXT_CLEAR_PAD_X, STATUS_TEXT_GAP_Y, STATUS_TEXT_MAX_CHARS,
};

// Single source of truth for token -> concrete u8g2 font mapping.
macro_rules! with_u8g2_font {
    ($font:expr, $renderer:ident, $body:block) => {{
        match $font {
            U8g2FontToken::Small => {
                let $renderer = FontRenderer::new::<fonts::u8g2_font_helvR08_te>()
                    .with_ignore_unknown_chars(true);
                $body
            }
            U8g2FontToken::Medium => {
                let $renderer = FontRenderer::new::<fonts::u8g2_font_helvR10_te>()
                    .with_ignore_unknown_chars(true);
                $body
            }
            U8g2FontToken::Large => {
                let $renderer = FontRenderer::new::<fonts::u8g2_font_helvR12_te>()
                    .with_ignore_unknown_chars(true);
                $body
            }
            U8g2FontToken::Larger => {
                let $renderer = FontRenderer::new::<fonts::u8g2_font_helvR18_te>()
                    .with_ignore_unknown_chars(true);
                $body
            }
        }
    }};
}

pub(super) fn font_for(font: FontToken) -> ResolvedFont {
    ResolvedFont::U8g2(font.into())
}

pub(super) fn text_width(font: ResolvedFont, text: &str) -> i32 {
    match font {
        ResolvedFont::U8g2(face) => u8g2_text_width(face, text),
    }
}

pub(super) fn centered_status_text_pos(font: ResolvedFont, text: &str) -> Point {
    let w = text_width(font, text);
    let x = GAUGE_CENTER.x - (w / 2);
    let y = GAUGE_CENTER.y + STATUS_TEXT_GAP_Y;
    Point::new(x, y)
}

pub(super) fn status_text_clear_rect(font: ResolvedFont) -> Rectangle {
    let max_chars = STATUS_TEXT_MAX_CHARS.max(1);
    let glyph_w = text_width(font, "0");
    let spacing = 1;
    let text_w = max_chars * glyph_w + (max_chars - 1) * spacing;
    let (text_y_offset, text_h) = status_text_bbox_metrics(font);

    let w = (text_w + STATUS_TEXT_CLEAR_PAD_X * 2).max(0) as u32;
    let h = (text_h + STATUS_TEXT_CLEAR_PAD_TOP + STATUS_TEXT_CLEAR_PAD_BOTTOM).max(0) as u32;
    let x = GAUGE_CENTER.x - (text_w / 2) - STATUS_TEXT_CLEAR_PAD_X;
    let y = GAUGE_CENTER.y + STATUS_TEXT_GAP_Y + text_y_offset - STATUS_TEXT_CLEAR_PAD_TOP;

    Rectangle::new(Point::new(x, y), Size::new(w, h))
}

pub(super) fn draw_text_aa<D>(
    display: &mut D,
    pos: Point,
    font: ResolvedFont,
    color: DisplayColor,
    text: &str,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    match font {
        ResolvedFont::U8g2(face) => draw_text_u8g2(display, pos, face, color, text),
    }
}

fn draw_text_u8g2<D>(
    display: &mut D,
    pos: Point,
    font: U8g2FontToken,
    color: DisplayColor,
    text: &str,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    with_u8g2_font!(font, renderer, {
        let _ = renderer.render(
            text,
            pos,
            VerticalPosition::Top,
            FontColor::Transparent(color),
            display,
        );
    });
}

fn u8g2_text_width(font: U8g2FontToken, text: &str) -> i32 {
    let dims = with_u8g2_font!(font, renderer, {
        renderer.get_rendered_dimensions(text, Point::zero(), VerticalPosition::Top)
    });

    dims.map(|d| d.advance.x.max(0)).unwrap_or(0)
}

fn u8g2_font_height(font: U8g2FontToken) -> i32 {
    with_u8g2_font!(font, renderer, {
        renderer.get_default_line_height() as i32
    })
}

fn u8g2_status_text_bbox_metrics(font: U8g2FontToken) -> (i32, i32) {
    with_u8g2_font!(font, renderer, {
        let sample_dims =
            renderer.get_rendered_dimensions("ÅHgjy", Point::zero(), VerticalPosition::Top);

        if let Ok(sample_dims) = sample_dims {
            if let Some(sample_bb) = sample_dims.bounding_box {
                let sample_top = sample_bb.top_left.y;
                let sample_bottom = sample_bb.top_left.y + sample_bb.size.height as i32;
                let font_top = renderer.get_glyph_bounding_box(VerticalPosition::Top).top_left.y;
                let top = sample_top.min(font_top);
                return (top, (sample_bottom - top).max(1));
            }
        }

        let fallback_h = renderer.get_default_line_height() as i32;
        let fallback_top = renderer.get_glyph_bounding_box(VerticalPosition::Top).top_left.y;
        (fallback_top, fallback_h.max(1))
    })
}

fn status_text_bbox_metrics(font: ResolvedFont) -> (i32, i32) {
    match font {
        ResolvedFont::U8g2(face) => u8g2_status_text_bbox_metrics(face),
    }
}

pub(super) fn font_height(font: ResolvedFont) -> i32 {
    match font {
        ResolvedFont::U8g2(face) => u8g2_font_height(face),
    }
}
