use iced::subscription;
use iced::Subscription;
use image::GenericImage;
use image::ImageBuffer;
use imageproc::stats::histogram;
use tokio::task::JoinHandle;
use std::default;
use std::ops;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use threadpool::ThreadPool;

use crate::Error;

pub type HistogramValueType = u32;
pub type Histogram = [HistogramValueType; 256];
pub type RgbHistogram = Vec<Histogram>;

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub path: PathBuf,
    pub error: Option<String>,
    pub histogram: Option<RgbHistogram>,
    pub checked: bool,
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            path: Default::default(),
            error: Default::default(),
            histogram: Default::default(),
            checked: Default::default(),
        }
    }
}

impl ops::Sub for ImageInfo {
    type Output = u32;

    fn sub(self, rhs: Self) -> Self::Output {
        if let (Some(hist_a), Some(hist_b)) = (self.histogram, rhs.histogram) {
            return hist_a
                .iter()
                .zip(hist_b)
                .map(|(chan_a, chan_b)| {
                    chan_a
                        .iter()
                        .zip(chan_b)
                        .fold(0, |acc, (value_a, value_b)| acc + (value_a - value_b))
                })
                .sum();
        }
        // Error: return max diff
        HistogramValueType::MAX
    }
}

pub fn analyze_new(paths: &'static [&Path]) -> Subscription<Progress> {
    subscription::unfold(0, State::Ready(paths), move |state| analyze(state))
}

pub async fn analyze(state: State) -> (Progress, State) {
    match state {
        State::Ready(paths) => {
            let mut analyzer = Analyse::new(paths);
            analyzer.start();
            (
                Progress::Started,
                State::Analyzing {
                    analyzer,
                    total: paths.len(),
                    progress: 0,
                },
            )
        }
        State::Analyzing {
            analyzer,
            total,
            progress: _,
        } => {
            let progress = analyzer.get_progress();
            let percentage = (progress as f32 / total as f32) * 100.0;
            if percentage == 100.0 {
                return (Progress::Finished, State::Finished);
            }
            (
                Progress::Advanced(percentage, "test".into()),
                State::Analyzing {
                    analyzer,
                    total,
                    progress,
                },
            )
        }
        State::Finished => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.
            iced::futures::future::pending().await
        }
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32, String),
    Finished,
    Errored,
}

pub enum State {
    Ready(&'static [&'static Path]),
    Analyzing {
        analyzer: Analyse,
        total: usize,
        progress: usize,
    },
    Finished,
}

#[derive(Default)]
struct Analyse {
    paths: &'static [&'static Path],
    progress: Arc<Mutex<usize>>,
    imageinfos: Arc<Mutex<Option<Vec<ImageInfo>>>>,
    receiver: Option<Receiver<ImageInfo>>,
    task: Option<JoinHandle<()>>,
}

impl Analyse {
    pub fn new(paths: &'static [&'static Path]) -> Self {
        Analyse {
            paths,
            ..Default::default()
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = channel::<ImageInfo>();
        self.receiver = Some(rx);
        self.task = Some(tokio::task::spawn(get_histograms(self.paths, tx)));
    }

    pub fn get_receiver(&self) -> Option<&Receiver<ImageInfo>> {
        self.receiver.as_ref()
    }

    pub fn get_progress(&self) -> usize {
        *self.progress.lock().unwrap()
    }

    pub fn get_total(&self) -> usize {
        self.paths.len()
    }
}

pub async fn get_histograms(paths: &'static [&Path], tx: Sender<ImageInfo>) {
    //let mut imageinfos = vec![];
    let cpu_count = num_cpus::get();
    let pool = ThreadPool::new(cpu_count);
    //let (tx, rx) = channel();

    for path in paths {
        let tx = tx.clone();
        pool.execute(move || {
            let imageinfo = get_imageinfo_from_image(path);
            tx.send(imageinfo)
                .expect("channel will be there waiting for the pool");
        });
    }

    // for imageinfo in rx.iter() {
    //     *self.progress.lock().unwrap() += 1;
    //     imageinfos.push(imageinfo);
    // }

    //Ok(imageinfos)
}

pub fn get_imageinfo_from_image(path: &Path) -> ImageInfo {
    let histograms = get_histograms_from_image(path);
    let mut imageinfo = ImageInfo {
        path: path.into(),
        ..Default::default()
    };
    match histograms {
        Ok(histograms) => imageinfo.histogram = Some(histograms),
        Err(error) => {
            imageinfo.error = match error {
                Error::LoadHistogram(msg) => Some(msg),
                _ => Some("unknown error".into()),
            }
        }
    };
    imageinfo
}

pub fn get_histograms_from_image(path: &Path) -> Result<RgbHistogram, Error> {
    match image::open(path) {
        Ok(img) => {
            let mut buffer = ImageBuffer::new(img.width(), img.height());
            buffer.copy_from(&img, 0, 0).unwrap();
            let histograms = histogram(&buffer);
            Ok(histograms.channels)
        }
        Err(error) => Err(Error::LoadHistogram(error.to_string())),
    }
}
