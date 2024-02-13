use eframe::egui;
use image::imageops::thumbnail;
use image::GenericImage;
use image::ImageBuffer;
use image::Rgba;
use imageproc::stats::histogram;
use std::hash::Hash;
use std::ops;
use std::ops::Deref;
use std::ops::Sub;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use threadpool::ThreadPool;
use tracing::debug;

use crate::gui::Message;
use crate::Error;

pub type HistogramValueType = u32;
#[derive(Debug, Clone)]
//pub type Histogram = [HistogramValueType; 256];
pub struct Histogram([HistogramValueType; 256]);

impl Deref for Histogram {
    type Target = [HistogramValueType; 256];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Iterator for Histogram {
    type Item = HistogramValueType;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

//pub type RgbHistogram = Vec<Histogram>;
#[derive(Debug, Clone)]
pub struct RgbHistogram(Vec<Histogram>);

impl Deref for RgbHistogram {
    type Target = Vec<Histogram>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Iterator for RgbHistogram {
    type Item = Histogram;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl Sub for RgbHistogram {
    type Output = u64;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut diff = 0;
        if self.len() != rhs.len() {
            return Self::Output::MAX;
        }
        for (color, _) in self.enumerate() {
            for (a, b) in self[color].zip(rhs[color]) {
                diff += (a as i64 - b as i64).abs() as u64;
            }
        }
        diff
    }
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub path: PathBuf,
    pub error: Option<String>,
    pub histogram: Option<RgbHistogram>,
    pub thumbnail: Vec<u8>,
    pub checked: bool,
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            path: Default::default(),
            error: Default::default(),
            histogram: Default::default(),
            thumbnail: vec![],
            checked: Default::default(),
        }
    }
}

impl PartialEq for ImageInfo {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for ImageInfo {}

impl Hash for ImageInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
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
}

pub fn get_imageinfo_from_image(path: PathBuf) -> ImageInfo {
    let mut imageinfo = ImageInfo {
        path: path.into(),
        ..Default::default()
    };

    // Load image file
    let img = match image::open(path) {
        Ok(img) => img,
        Err(error) => {
            imageinfo.error = Some(format!("{error:?}"));
            return imageinfo;
        }
    };

    // Load histogram
    let mut buffer = ImageBuffer::new(img.width(), img.height());
    buffer.copy_from(&img, 0, 0).unwrap();
    let histograms = histogram(&buffer);
    imageinfo.histogram = Some(histograms.channels);

    // Create thumbnail
    let thumb = thumbnail(&img, 100, 100);
    imageinfo.thumbnail = thumb.into_vec();

    imageinfo
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
            tx.send(Message::ThumbnailCreated((
                path.display().to_string(),
                image_buffer,
            )))
            .expect("channel will be there waiting for the pool");
            repaint_signal.request_repaint();
        });
    }
    pool.join();
    tx.send(Message::ThumbnailsCreated)
        .expect("Message not sent");
    context.request_repaint();
}

fn get_thumbnail(path: &PathBuf) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Error> {
    debug!("get_thumbnail {}", path.display());
    let img = image::open(path)?;
    let image_buffer = thumbnail(&img, 100, 100);
    Ok(image_buffer)
}
