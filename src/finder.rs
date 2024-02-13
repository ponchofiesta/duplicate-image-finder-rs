use crate::{image::ImageInfo, Error};
use std::{collections::HashSet, ops::{Deref, DerefMut, Sub}, path::{Path, PathBuf}};
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
    let pairs = images
        .iter()
        .zip(images)
        .filter(|(a, b)| !std::ptr::eq(a, b) && a.histogram.is_some() && b.histogram.is_some())
        .map(|(a, b)| Pair::new(a, b, a.histogram.as_ref().unwrap().diff(b.histogram.as_ref().unwrap())))
        .collect();
    pairs
}

pub fn get_groups<'a>(pairs: &'a Vec<Pair<'a>>) -> Vec<ImageInfoGroup<'a>> {
    let mut groups: Vec<ImageInfoGroup> = vec![];
    for pair in pairs {
        let mut pair_in_groups = vec![];

        // Search items in all groups
        for (i, group) in groups.iter().enumerate() {
            if group.contains(&pair.a) || group.contains(&pair.b) {
                pair_in_groups.push(i);
            }
        }

        // If matching items were found in multiple groups, merge those groups
        if pair_in_groups.len() > 1 {
            for group_id in pair_in_groups.iter().skip(1).rev() {
                let group = groups[*group_id].clone();
                groups[pair_in_groups[0]].extend(group.iter());
                groups.remove(*group_id);
            }
        }

        // Add items to the groups
        if pair_in_groups.len() > 0 {
            groups[pair_in_groups[0]].extend([pair.a, pair.b]);
        } else {
            groups.push(ImageInfoGroup::from_vec(&[pair.a, pair.b]));
        }
    }
    
    groups
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

pub struct ImageInfoGroup<'a>(HashSet<&'a ImageInfo>);

impl<'a> Deref for ImageInfoGroup<'a> {
    type Target = HashSet<&'a ImageInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for ImageInfoGroup<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> ImageInfoGroup<'a> {
    pub fn new() -> Self {
        ImageInfoGroup(HashSet::new())
    }

    pub fn from_vec(values: &'a [&'a ImageInfo]) -> Self {
        let mut group = ImageInfoGroup::new();
        group.extend(values);
        group
    }
}