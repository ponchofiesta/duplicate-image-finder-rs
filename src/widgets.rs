use eframe::{
    egui::{Image, ProgressBar, Ui},
    epaint::Vec2,
};
use image::{imageops::thumbnail, ImageBuffer};

#[derive(PartialEq)]
pub struct Progress {
    pub value: u64,
    pub total: u64,
    pub message: String,
}

impl Progress {
    pub fn ratio(&self) -> f32 {
        self.value as f32 / self.total as f32
    }
}

impl ToString for Progress {
    fn to_string(&self) -> String {
        format!("{} ({}/{})", self.message, self.value, self.total)
    }
}

pub fn progress(ui: &mut Ui, progress: &Progress) {
    ui.vertical(|ui| {
        ui.add(ProgressBar::new(progress.ratio()));
        ui.label(progress.to_string());
    });
}

pub fn selectable_image(ui: &mut Ui, path: &str, checked: &mut bool, text: &str) {
    ui.vertical(|ui| {
        let url = format!("file://{}", path);
        ui.add(Image::new(url).max_size(Vec2::new(100., 100.)));
        ui.checkbox(checked, text);
    });
}

// pub fn image_thumbnail(ui: &mut Ui, path: &str, max_size: Vec2) {
//     match image::open(path) {
//         Ok(img) => {
//             //let mut buffer = ImageBuffer::new(100, 100);
//             //buffer.copy_from(&img, 0, 0).unwrap();
//             let thumb = thumbnail(&img, 100, 100);
//             ui.add()
//         }
//         Err(error) => Err(Error::LoadHistogram(error.to_string())),
//     }
//     Image::new(url).max_size(Vec2::new(100., 100.))
// }