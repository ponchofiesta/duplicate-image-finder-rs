use std::io;

use gui::DupApp;
use iced::{Application, Settings};

mod finder;
mod gui;
mod image;
mod widgets;
mod multiprocessing;

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
