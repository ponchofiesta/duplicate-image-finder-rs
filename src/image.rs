use iced::Subscription;
use iced::subscription;
use image::GenericImage;
use image::ImageBuffer;
use imageproc::stats::histogram;
use std::borrow::Borrow;
use std::ops;
use std::path::Path;
use std::path::PathBuf;

use crate::Error;
use crate::gui::Message;
use crate::multiprocessing;

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
        Self { path: Default::default(), error: Default::default(), histogram: Default::default(), checked: Default::default() }
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

pub fn analyze_new(paths: &[&Path]) -> Subscription<Progress> {
    subscription::unfold(0, State::Ready(paths), move |state| {
        analyze(state)
    })
}

pub async fn analyze<'a>(state: State<'a>) -> (Progress, State<'a>) {
    match state {
        State::Ready(paths) => {
            let response = get_histograms(paths).await;

            match response {
                Ok(images) => (
                    Progress::Started,
                    State::Analyzing {
                        total: images.len(),
                        progress: 0,
                    },
                ),
                Err(_) => (Progress::Errored, State::Finished),
            }
        },
        State::Analyzing { total, progress } => match response.chunk().await {
            Ok(Some(chunk)) => {
                let progress = progress + chunk.len() as u64;

                let percentage = (progress as f32 / total as f32) * 100.0;

                (
                    Progress::Advanced(percentage, "test".into()),
                    State::Analyzing { total, progress },
                )
            }
            Ok(None) => (Progress::Finished, State::Finished),
            Err(_) => (Progress::Errored, State::Finished),
        },
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

pub enum State<'a> {
    Ready(&'a [&'a Path]),
    Analyzing { total: usize, progress: usize },
    Finished,
}

struct Analyse {

}

pub async fn get_histograms(paths: &[&Path]) -> Result<Vec<ImageInfo>, Error> {
    let mut imageinfos = vec![];
    let pool = multiprocessing::ThreadPool::new();
    for result in pool.imap(get_imageinfo_from_image, paths) {
        imageinfos.push(result);
    }
    Ok(imageinfos)
}

pub fn get_imageinfo_from_image(path: &Path) -> ImageInfo {
    let histograms = get_histograms_from_image(path);
    let mut imageinfo = ImageInfo {
        path: path.into(),
        ..Default::default()
    };
    match histograms {
        Ok(histograms) => imageinfo.histogram = Some(histograms),
        Err(error) => imageinfo.error = match error {
            Error::LoadHistogram(msg) => Some(msg),
            _ => Some("unknown error".into()),
        },
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
