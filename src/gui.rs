use crate::finder::ImageInfoGroup;
use crate::image::ImageInfo;
use crate::widgets::Progress;
use crate::{finder, image};
use crate::{widgets, Error};

use eframe::egui::{self, Layout};
use eframe::App;
use ::image::{ImageBuffer, Rgba};

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tracing::debug;

pub struct DupApp<'a> {
    context: egui::Context,
    state: State,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    folder_path: Option<PathBuf>,
    image_paths: Vec<PathBuf>,
    images: Vec<ImageInfo>,
    thumbnails: HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    groups: Vec<ImageInfoGroup<'a>>,
    //analyze: Option<Analyze>,
    // error: Option<Error>,
}

fn load_icon_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "icons".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/icons.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "icons".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("icons".to_owned());
    ctx.set_fonts(fonts);
}

impl<'a> DupApp<'a> {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_icon_font(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let context = cc.egui_ctx.clone();
        let (sender, receiver) = channel();
        DupApp {
            context,
            state: State::Open,
            sender,
            receiver,
            folder_path: None,
            image_paths: vec![],
            images: vec![],
            thumbnails: HashMap::new(),
            groups: vec![],
        }
    }

    fn handle_messages(&mut self) {
        if let Ok(msg) = self.receiver.try_recv() {
            match msg {
                Message::FolderOpened(path) => {
                    debug!("Message::FolderOpened({path:?})");
                    self.folder_path = path;
                    self.search_images();
                }
                Message::ImagesFound(paths) => {
                    debug!("Message::ImagesFound({paths:?})");
                    match paths {
                        Ok(paths) => {
                            self.image_paths = paths;
                            self.analyze_images();
                        }
                        Err(_e) => todo!(),
                    }
                }
                Message::ImageAnalyzed(image_info) => {
                    debug!("Message::ImageAnalyzed({image_info:?})");
                    self.images.push(image_info);
                    if let State::Analyzing(progress) = self.state.borrow_mut() {
                        progress.value += 1;
                    }
                }
                Message::ImagesAnalyzed => {
                    debug!("Message::ImagesAnalyzed");
                    self.state = State::Analyzed;
                }
                Message::ThumbnailCreated((path, thumbnail)) => {
                    let thumbnail = match thumbnail {
                        Ok(thumbnail) => thumbnail,
                        // TODO: include error image
                        Err(_e) => ImageBuffer::new(100, 100),
                    };
                    self.thumbnails.insert(path, thumbnail);
                    if let State::CreatingThumbnails(progress) = self.state.borrow_mut() {
                        progress.value += 1;
                    }
                },
                Message::ThumbnailsCreated => {
                    self.state = State::Select;
                },
            }
        };
    }

    fn open_folder(&mut self) {
        let path = rfd::FileDialog::new()
            .set_title("Open image folder")
            .pick_folder();
        self.sender
            .send(Message::FolderOpened(path))
            .expect("Message not sent");
    }

    fn search_images(&mut self) {
        self.state = State::SearchingImages(widgets::Progress {
            value: 0,
            total: 1,
            message: "Searching for images".into(),
        });
        if let Some(ref path) = self.folder_path {
            let path = path.clone();
            let tx = self.sender.clone();
            let context = self.context.clone();
            thread::spawn(move || {
                let result = finder::find_images(path);
                tx.send(Message::ImagesFound(result))
                    .expect("Message not sent");
                context.request_repaint();
            });
        }
    }

    fn analyze_images(&mut self) {
        self.state = State::Analyzing(widgets::Progress {
            value: 0,
            total: self.image_paths.len() as u64,
            message: "Analyzing images".into(),
        });
        let tx = self.sender.clone();
        let paths = self.image_paths.clone();
        let context = self.context.clone();
        thread::spawn(move || {
            image::get_histograms(&paths, tx, &context);
            // let pairs = finder::compare_images(images, 10_000_000);
            // let groups = finder::get_groups(&pairs);
            context.request_repaint();
        });
    }

    fn select_images(&mut self) {
        self.state = State::CreatingThumbnails(Progress {
            value: 0,
            total: self.images.len() as u64,
            message: "Creating thumbnails".into(),
        });
        let tx = self.sender.clone();
        let paths = self.image_paths.clone();
        let context = self.context.clone();
        thread::spawn(move || {
            image::get_thumbnails(&paths, tx, &context);
            context.request_repaint();
        });
    }

    fn render(&mut self, ctx: &egui::Context) {
        match self.state {
            State::Analyzing(ref progress) | State::CreatingThumbnails(ref progress) => {
                egui::CentralPanel::default().show(&ctx, |ui| {
                    widgets::progress(ui, progress);
                });
            }
            State::Select => {
                egui::CentralPanel::default().show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        self.images.iter_mut().for_each(|imageinfo| {
                            widgets::selectable_image(ui, imageinfo)
                        });
                    });
                });
            }
            _ => {
                egui::CentralPanel::default().show(&ctx, |ui| {
                    ui.with_layout(Layout::left_to_right(eframe::emath::Align::Min), |ui| {
                        let open_folder_button = ui.button("\u{E800}");
                        if open_folder_button.clicked() {
                            self.open_folder();
                        }

                        match self.folder_path.as_deref().and_then(Path::to_str) {
                            Some(path) => ui.label(path),
                            None => ui.label("-"),
                        };
                    });
                    ui.with_layout(Layout::left_to_right(eframe::emath::Align::Min), |ui| {
                        let select_button = ui.button("Select");
                        if select_button.clicked() {
                            self.select_images();
                        }
                    });
                });
            }
        };
    }
}

impl<'a> App for DupApp<'a> {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();
        self.render(ctx);
    }
}

#[derive(PartialEq)]
pub enum State {
    Open,
    SearchingImages(widgets::Progress),
    Analyzing(widgets::Progress),
    Analyzed,
    CreatingThumbnails(widgets::Progress),
    ThumbnailsCreated,
    Select,
    // Delete,
    // Deleting,
}

impl Default for State {
    fn default() -> Self {
        State::Open
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // FontLoaded(Result<(), font::Error>),
    // FolderOpen,
    FolderOpened(Option<PathBuf>),
    ImagesFound(Result<Vec<PathBuf>, Error>),
    ImageAnalyzed(ImageInfo),
    ImagesAnalyzed,
    ThumbnailCreated((String, Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Error>)),
    ThumbnailsCreated,
    // Analyse,
    // AnalyseProgressed(image::Progress),
    // ImagesLoaded(Result<Vec<ImageInfo>, Error>),
}
