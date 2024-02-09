use eframe::egui;
use image::imageops::thumbnail;
use image::GenericImage;
use image::ImageBuffer;
use image::ImageResult;
use image::Rgba;
use imageproc::stats::histogram;
use tracing::debug;
use std::io;
use std::ops;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use threadpool::ThreadPool;

use crate::gui::Message;
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

pub fn get_histograms(paths: &[PathBuf], tx: Sender<Message>, context: &egui::Context) {
    let cpu_count = num_cpus::get();
    let pool = ThreadPool::new(cpu_count);

    for path in paths {
        let path = path.clone();
        let tx = tx.clone();
        let repaint_signal = context.clone();
        pool.execute(move || {
            let imageinfo = get_imageinfo_from_image(path);
            tx.send(Message::ImageAnalyzed(imageinfo))
                .expect("channel will be there waiting for the pool");
            repaint_signal.request_repaint();
        });
    }
    pool.join();
    tx.send(Message::ImagesAnalyzed).expect("Message not sent");
    context.request_repaint();
}

pub fn get_imageinfo_from_image(path: PathBuf) -> ImageInfo {
    let histograms = get_histograms_from_image(&path);
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

pub fn get_thumbnails(paths: &[PathBuf], tx: Sender<Message>, context: &egui::Context) {
    let cpu_count = num_cpus::get();
    let pool = ThreadPool::new(cpu_count);

    for path in paths {
        let path = path.clone();
        let tx = tx.clone();
        let repaint_signal = context.clone();
        pool.execute(move || {
            let image_buffer = get_thumbnail(&path);
            tx.send(Message::ThumbnailCreated((path.display().to_string(), image_buffer)))
                .expect("channel will be there waiting for the pool");
            repaint_signal.request_repaint();
        });
    }
    pool.join();
    tx.send(Message::ThumbnailsCreated).expect("Message not sent");
    context.request_repaint();
}

fn get_thumbnail(path: &PathBuf) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Error> {
    debug!("get_thumbnail {}", path.display());
    let img = image::open(path)?;
    let image_buffer = thumbnail(&img, 100, 100);
    Ok(image_buffer)
}