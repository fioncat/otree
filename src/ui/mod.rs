use ratatui::style::Style;
use ratatui::widgets::BorderType;

use crate::config::colors::Color;

mod app;
mod data_block;
mod header;
mod tree_overview;

pub use app::App;
pub use header::HeaderContext;

fn get_border_style(focus_color: &Color, normal_color: &Color, focus: bool) -> (Style, BorderType) {
    let color = if focus { focus_color } else { normal_color };
    let border_type = if color.bold {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    (color.style, border_type)
}
