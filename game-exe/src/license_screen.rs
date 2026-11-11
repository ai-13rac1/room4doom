//! The license screen: the BuildDeed license state in the Doom font, in a box
//! at the bottom of the title screen and the demos. R and B there register
//! this computer and open the buy page (see `d_main::input_responder`).

use gamestate::Game;
use render_common::DrawBuffer;
use ui_draw::{UiCanvas, measure_text_line};

/// The artwork size of the Doom screen.
const WIDTH: f32 = 320.0;
const HEIGHT: f32 = 200.0;
const MARGIN: f32 = 4.0;
const LINE_HEIGHT: f32 = 10.0;
const BOX_COLOUR: u32 = 0xFF00_0000;

/// Draws the license lines, centered, over a black box.
#[expect(clippy::cast_precision_loss, reason = "the box has a few lines")]
pub(crate) fn draw(game: &Game, buf: &mut impl DrawBuffer) {
    let lines: Vec<String> = game
        .license
        .lines()
        .iter()
        .flat_map(|l| {
            wrap(&font_text(l), WIDTH - 4.0 * MARGIN, |t| {
                measure_text_line(t, 1.0)
            })
        })
        .collect();
    let top = HEIGHT - MARGIN - LINE_HEIGHT * lines.len() as f32 - MARGIN;
    let viewport = buf.ui_layout().centered();
    let [x0, y0] = viewport.point(MARGIN, top);
    let [x1, y1] = viewport.point(WIDTH - MARGIN, HEIGHT - MARGIN);
    fill(buf, [x0, y0], [x1, y1]);
    let mut canvas = UiCanvas::new(buf, game.pic_data.wad_palette(), viewport);
    for (i, line) in lines.iter().enumerate() {
        let x = (WIDTH - measure_text_line(line, 1.0)) / 2.0;
        canvas.text(line, [x, top + MARGIN / 2.0 + LINE_HEIGHT * i as f32]);
    }
}

/// The HUD font has the ASCII characters from '!' to '_' (no lower case
/// letters). Other characters become spaces.
fn font_text(line: &str) -> String {
    line.chars()
        .map(|c| match c.to_ascii_uppercase() {
            u @ '!'..='_' => u,
            _ => ' ',
        })
        .collect()
}

/// Fills the rectangle from `a` to `b` (buffer pixels) with the box colour.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "screen coordinates are quantized to pixels; negative values clamp to 0"
)]
fn fill(buf: &mut impl DrawBuffer, a: [f32; 2], b: [f32; 2]) {
    let width = buf.size().width_usize();
    let height = buf.size().height_usize();
    let (x0, x1) = (a[0].max(0.0) as usize, (b[0].max(0.0) as usize).min(width));
    let (y0, y1) = (a[1].max(0.0) as usize, (b[1].max(0.0) as usize).min(height));
    for y in y0..y1 {
        for x in x0..x1 {
            buf.set_pixel(x, y, BOX_COLOUR);
        }
    }
}

/// Splits a line at spaces so that each part is at most `max` wide by
/// `width`. A word wider than `max` stays on its own line.
fn wrap(line: &str, max: f32, width: impl Fn(&str) -> f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for word in line.split(' ') {
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if width(&candidate) <= max || current.is_empty() {
            current = candidate;
        } else {
            out.push(std::mem::replace(&mut current, word.to_owned()));
        }
    }
    out.push(current);
    out
}

#[cfg(test)]
mod tests {
    use super::{font_text, wrap};

    #[test]
    fn font_text_has_only_font_characters() {
        assert_eq!(
            font_text("Registered to a_b.testnet"),
            "REGISTERED TO A_B.TESTNET"
        );
        assert_eq!(font_text("caf\u{e9} {x}\n"), "CAF   X  ");
    }

    #[expect(clippy::cast_precision_loss, reason = "short test strings")]
    fn chars(t: &str) -> f32 {
        t.len() as f32
    }

    #[test]
    fn wrap_splits_at_spaces() {
        assert_eq!(wrap("aaa bbb ccc", 7.0, chars), vec!["aaa bbb", "ccc"]);
        assert_eq!(wrap("short", 7.0, chars), vec!["short"]);
        assert_eq!(wrap("toolongword x", 4.0, chars), vec!["toolongword", "x"]);
    }
}
