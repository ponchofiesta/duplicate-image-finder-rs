use iced::{
    widget::{button, container, row, text, Button},
    Element, Font,
};

use crate::gui::Message;

pub fn icon<'a>(codepoint: char) -> Element<'a, Message> {
    const FONT: Font = Font::with_name("icons");
    text(codepoint).font(FONT).into()
}

pub fn icon_button<'a>(codepoint: char, label: &str) -> Button<'a, Message> {
    button(container(row![container(icon(codepoint)), text(label)].spacing(8)).padding([4, 8]))
        .into()
}
