mod finder;
mod gui;
mod image;
mod widgets;

use eframe::egui;
use gui::DupApp;
use ::image::ImageError;
use std::io;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    NoImageFound,
    Io(String),
    ImageLoad(String),
    Image(String),
}

impl From<ImageError> for Error {
    fn from(value: ImageError) -> Self {
        Error::Image(value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value.to_string())
    }
}

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    debug!("hallo");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Duplicate Image Finder",
        options,
        Box::new(|cc| Box::new(DupApp::new(cc))),
    )
}
