use crate::image;
use crate::image::ImageInfo;
use crate::widgets::{icon, icon_button};
use crate::Error;
use iced::{
    self, executor, font,
    widget::{button, column, container, row, text},
    Application, Command, Element, Settings, Theme,
};
use iced::{subscription, Subscription};
use std::sync::{Arc, Mutex};
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct DupApp {
    state: State,
    folder_path: Option<PathBuf>,
    analyze: Option<Analyze>,
    error: Option<Error>,
}

pub enum State {
    Open,
    Select,
    Analyzing,
    Delete,
    Deleting,
}

impl Default for State {
    fn default() -> Self {
        State::Open
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    FontLoaded(Result<(), font::Error>),
    FolderOpen,
    FolderOpened(Result<PathBuf, Error>),
    Analyse,
    AnalyseProgressed(image::Progress),
    ImagesLoaded(Result<Vec<ImageInfo>, Error>),
}

impl Application for DupApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            DupApp::default(),
            Command::batch(vec![font::load(
                include_bytes!("../assets/icons.ttf").as_slice(),
            )
            .map(Message::FontLoaded)]),
        )
    }

    fn title(&self) -> String {
        "Duplicate Image Finder".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::FolderOpen => Command::perform(open_folder(), Message::FolderOpened),
            Message::FolderOpened(path) => match path {
                Ok(path) => {
                    self.folder_path = Some(path);
                    Command::perform(
                        load_folder(self.folder_path.as_ref().unwrap().clone()),
                        Message::ImagesLoaded,
                    )
                }
                Err(error) => {
                    self.error = Some(error);
                    Command::none()
                }
            },
            Message::Analyse => {
                if let Some(ref mut analyze) = self.analyze {
                    analyze.start();
                }
                Command::none()
            }
            Message::AnalyseProgressed(progress) => {
                if let Some(ref mut analyze) = self.analyze {
                    analyze.progress(progress);
                }
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Some(ref analyze) = self.analyze {
            return Subscription::batch(vec![analyze.subscription()]);
        }
        Subscription::none()
    }

    fn view(&self) -> Element<Self::Message> {
        container(column!(row!(
            icon_button('\u{E800}', "Open folder").on_press(Message::FolderOpen),
            match self.folder_path.as_deref().and_then(Path::to_str) {
                Some(path) => text(path),
                None => text("-"),
            }
        )))
        .padding(10)
        .into()
    }
}

async fn open_folder() -> Result<PathBuf, Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Open image folder")
        .pick_folder()
        .await
        .ok_or(Error::DialogClosed)?;
    Ok(handle.path().to_owned())
}

async fn load_folder(path: PathBuf) -> Result<Vec<ImageInfo>, Error> {
    Ok(vec![])
    //Err(Error::NoImageFound)
}

#[derive(Debug)]
enum AnalyseState {
    Idle,
    Analyzing { progress: f32 },
    Finished,
    Errored,
}

pub struct Analyze {
    paths: &'static [&'static Path],
    state: AnalyseState,
}

impl Analyze {
    pub fn total(&self) -> usize {
        self.paths.len()
    }

    pub fn start(&mut self) {
        match self.state {
            AnalyseState::Idle { .. }
            | AnalyseState::Finished { .. }
            | AnalyseState::Errored { .. } => {
                self.state = AnalyseState::Analyzing { progress: 0.0 };
            }
            AnalyseState::Analyzing { .. } => {}
        }
    }

    pub fn progress(&mut self, new_progress: image::Progress) {
        if let AnalyseState::Analyzing { progress } = &mut self.state {
            match new_progress {
                image::Progress::Started => {
                    *progress = 0.0;
                }
                image::Progress::Advanced(percentage, msg) => {
                    *progress = percentage;
                }
                image::Progress::Finished => {
                    self.state = AnalyseState::Finished;
                }
                image::Progress::Errored => {
                    self.state = AnalyseState::Errored;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            AnalyseState::Analyzing { .. } => {
                image::analyze_new(self.paths).map(Message::AnalyseProgressed)
            }
            _ => Subscription::none(),
        }
    }
}
