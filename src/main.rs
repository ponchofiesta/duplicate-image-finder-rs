use iced::{
    self, executor,
    widget::{button, column, container, row, text},
    Application, Command, Element, Settings, Theme,
};
use std::path::{Path, PathBuf};

#[derive(Default)]
struct DupApp {
    state: State,
    path: Option<PathBuf>,
}

enum State {
    Open,
    Select,
    Analyzing,
    Delete,
}

impl Default for State {
    fn default() -> Self {
        State::Open
    }
}

#[derive(Debug, Clone)]
enum Message {
    OpenFolder,
    FolderOpened(Result<PathBuf, Error>),
}

impl Application for DupApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (DupApp::default(), Command::none())
    }

    fn title(&self) -> String {
        "Duplicate Image Finder".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::OpenFolder => Command::perform(open_folder(), Message::FolderOpened),
            Message::FolderOpened(path) => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        container(column!(row!(
            button("Open").on_press(Message::OpenFolder),
            match self.path.as_deref().map(Path::to_str) {
                Some(path) => text(path),
                None => text("-"),
            }
        )))
        .into()
    }
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
}

async fn open_folder() -> Result<PathBuf, Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Open image folder")
        .pick_folder()
        .await
        .ok_or(Error::DialogClosed)?;
    Ok(handle.path().to_owned())
}

fn main() -> iced::Result {
    DupApp::run(Settings::default())
}
