mod finder;
mod gui;
mod image;
mod util;
mod widgets;

use gui::DupApp;
use iced::{Application, Settings};
use std::io;

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    NoImageFound,
    Io(String),
    LoadHistogram(String),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value.to_string())
    }
}

fn main() -> iced::Result {
    DupApp::run(Settings::default())
}
