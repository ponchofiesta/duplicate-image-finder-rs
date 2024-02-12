use crate::{image::ImageInfo, Error};
use std::path::{Path, PathBuf};
use walkdir;

pub fn find_images<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Error> {
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

pub fn compare_images<'a>(images: &'a [&'a ImageInfo]) -> Vec<Pair<'a>> {
    let mut pairs = vec![];
    images
        .iter()
        .zip(images)
        .filter(|(a, b)| !std::ptr::eq(a, b))
        .map(|(a, b)| Pair::new(a, b, a.histogram - b.histogram))

    // pairs = []
    // for i, a in enumerate(histograms):
    //     for b in histograms[i+1:]:
    //         pair = Pair(a=a, b=b, diff=None)
    //         pairs.append(pair)

    // # Get all diffs
    // status = "Comparing files..."
    // self._progress_handler(0, f"{status} (2/2)")
    // diffs = []
    // with ThreadPool() as pool:
    //     total = len(pairs)
    //     for i, diff in enumerate(pool.imap_unordered(self.get_diff, pairs)):
    //         if self.cancel:
    //             pool.terminate()
    //             return ([], [])
    //         self._progress_handler(int(i / total * 100), f"{status} (2/2)")
    //         if diff.diff is not None and diff.diff < threshold:
    //             diffs.append(diff)

    // groups = self.get_groups(diffs)
}

pub struct Pair<'a> {
    a: &'a ImageInfo,
    b: &'a ImageInfo,
    diff: u64,
}

impl<'a> Pair<'a> {
    pub fn new(a: &'a ImageInfo, b: &'a ImageInfo, diff: u64) -> Self {
        Pair { a, b, diff }
    }
}

pub struct ImageInfoGroup {}
