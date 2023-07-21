use std::{
    fs::{write, File},
    io::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::Result;
pub use paperdoll;
use paperdoll::{factory::PaperdollFactory, manifest::Manifest};
use tar::{Archive, Builder};
use tempdir::TempDir;

pub const EXTENSION_NAME: &'static str = "ppd";

pub const FILE_NAME_MANIFEST: &'static str = "manifest.yml";
pub const FILE_NAME_TEMP_DIR: &'static str = "paperdoll_ppd_";

pub fn load<P>(path: P) -> Result<PaperdollFactory>
where
    P: AsRef<Path>,
{
    let file = File::open(&path)?;
    let mut archive = Archive::new(file);

    let temp_dir = TempDir::new(FILE_NAME_TEMP_DIR)?;
    archive.unpack(temp_dir.path())?;

    let manifest_path = temp_dir.path().clone().join(FILE_NAME_MANIFEST);
    let manifest_file = File::open(manifest_path)?;

    let mut manifest: Manifest = serde_yaml::from_str(&read_to_string(manifest_file)?)?;

    for doll in &mut manifest.dolls {
        if doll.path.is_empty() {
            continue;
        }

        let img_path = temp_dir.path().clone().join(&doll.path);
        let img = image::open(img_path)?.into_rgba8();

        doll.image.width = img.width();
        doll.image.height = img.height();
        doll.image.pixels = img.into_vec();
    }

    for fragment in &mut manifest.fragments {
        if fragment.path.is_empty() {
            continue;
        }

        let img_path = temp_dir.path().clone().join(&fragment.path);
        let img = image::open(img_path)?.into_rgba8();

        fragment.image.width = img.width();
        fragment.image.height = img.height();
        fragment.image.pixels = img.into_vec();
    }

    temp_dir.close()?;

    PaperdollFactory::from_manifest(manifest)
}

pub fn save<P>(manifest: &Manifest, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let temp_dir = TempDir::new(FILE_NAME_TEMP_DIR)?;

    let manifest_str = serde_yaml::to_string(manifest)?;

    let manifest_path = temp_dir.path().clone().join(FILE_NAME_MANIFEST);
    write(manifest_path, manifest_str)?;

    for doll in &manifest.dolls {
        if doll.image.is_empty() {
            continue;
        }

        let filename = if doll.path.is_empty() {
            format!("{}.png", doll.id())
        } else {
            PathBuf::from(&doll.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        };

        let img_path = temp_dir.path().clone().join(filename);

        image::save_buffer(
            img_path,
            &doll.image.pixels,
            doll.image.width,
            doll.image.height,
            image::ColorType::Rgba8,
        )?;
    }

    for fragment in &manifest.fragments {
        if fragment.image.is_empty() {
            continue;
        }

        let filename = if fragment.path.is_empty() {
            format!("{}.png", fragment.id())
        } else {
            PathBuf::from(&fragment.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        };

        let img_path = temp_dir.path().clone().join(filename);

        image::save_buffer(
            img_path,
            &fragment.image.pixels,
            fragment.image.width,
            fragment.image.height,
            image::ColorType::Rgba8,
        )?;
    }

    let output = File::create(path)?;

    let mut archive = Builder::new(output);

    archive.append_dir_all(".", temp_dir.path())?;

    archive.finish()?;

    temp_dir.close()?;

    Ok(())
}
