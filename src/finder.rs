use std::{path::{PathBuf, Path}};

use walkdir;

use crate::{image::{RgbHistogram, ImageInfo}, Error};



pub async fn find_images<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Error> {
    const EXTENSIONS: [&'static str; 5] = [".jpg", ".jpeg", ".png", ".bmp", ".gif"];
    let files = walkdir::WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_owned())
        .filter(|entry| {
            let name = entry.to_string_lossy().to_lowercase();
            EXTENSIONS.iter().any(|ext| name.ends_with(ext))
        })
        .collect();
    Ok(files)
}

pub async fn load_images(paths: &Vec<PathBuf>) -> Result<Vec<ImageInfo>, Error> {
    todo!()
}
