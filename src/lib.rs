//! [Tar](https://en.wikipedia.org/wiki/Tar_%28computing%29) archive container format for [paperdoll](https://github.com/fralonra/paperdoll).
//!
//! # Examples
//!
//! ```no_run
//! use paperdoll_tar::paperdoll;
//!
//! let factory = paperdoll::PaperdollFactory::default();
//!
//! paperdoll_tar::save(&mut factory.to_manifest(), "/path/to/save/your.ppd");
//!
//! let factory = paperdoll_tar::load("/path/to/save/your.ppd").unwrap();
//! ```

use std::{
    fs::{write, File},
    io::{read_to_string, Read},
    path::{Path, PathBuf},
};

use anyhow::Result;
pub use paperdoll;
use paperdoll::{Manifest, PaperdollFactory};
use tar::{Archive, Builder};
use tempdir::TempDir;

/// The file extension.
pub const EXTENSION_NAME: &'static str = "ppd";

/// The file name of the manifest file saved in the `ppd` file.
pub const FILE_NAME_MANIFEST: &'static str = "manifest.yml";

const FILE_NAME_TEMP_DIR: &'static str = "paperdoll_ppd_";

/// Loads a paperdoll project from the path of a `ppd` file.
pub fn load<P>(path: P) -> Result<PaperdollFactory>
where
    P: AsRef<Path>,
{
    read(File::open(&path)?)
}

/// Reads a paperdoll project from a reader containing the bytes of a `ppd` file.
pub fn read<R>(r: R) -> Result<PaperdollFactory>
where
    R: Read,
{
    let mut archive = Archive::new(r);

    let temp_dir = TempDir::new(FILE_NAME_TEMP_DIR)?;
    archive.unpack(temp_dir.path())?;

    let manifest_path = temp_dir.path().join(FILE_NAME_MANIFEST);
    let manifest_file = File::open(manifest_path)?;

    let mut manifest: Manifest = serde_yaml::from_str(&read_to_string(manifest_file)?)?;

    for doll in &mut manifest.dolls {
        if doll.path.is_empty() {
            continue;
        }

        let img_path = temp_dir.path().join(&doll.path);
        let img = image::open(img_path)?.into_rgba8();

        doll.image.width = img.width();
        doll.image.height = img.height();
        doll.image.pixels = img.into_vec();
    }

    for fragment in &mut manifest.fragments {
        if fragment.path.is_empty() {
            continue;
        }

        let img_path = temp_dir.path().join(&fragment.path);
        let img = image::open(img_path)?.into_rgba8();

        fragment.image.width = img.width();
        fragment.image.height = img.height();
        fragment.image.pixels = img.into_vec();
    }

    temp_dir.close()?;

    PaperdollFactory::from_manifest(manifest)
}

/// Saves a `ppd` file using the given manifest to the path.
pub fn save<P>(manifest: &mut Manifest, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let temp_dir = TempDir::new(FILE_NAME_TEMP_DIR)?;

    for doll in &mut manifest.dolls {
        if doll.image.is_empty() {
            continue;
        }

        let extension = (!doll.path.is_empty())
            .then(|| {
                PathBuf::from(&doll.path)
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string())
            })
            .flatten()
            .map(|ext| {
                // Saving WebP is not supported
                if &ext == "webp" {
                    "png".to_owned()
                } else {
                    ext
                }
            })
            .unwrap_or("png".to_owned());

        let filename = format!("doll_{}.{}", doll.id(), extension);

        let img_path = temp_dir.path().join(&filename);

        doll.path = filename;

        image::save_buffer(
            img_path,
            &doll.image.pixels,
            doll.image.width,
            doll.image.height,
            image::ColorType::Rgba8,
        )?;
    }

    for fragment in &mut manifest.fragments {
        if fragment.image.is_empty() {
            continue;
        }

        let extension = (!fragment.path.is_empty())
            .then(|| {
                PathBuf::from(&fragment.path)
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string())
            })
            .flatten()
            .map(|ext| {
                // Saving WebP is not supported
                if &ext == "webp" {
                    "png".to_owned()
                } else {
                    ext
                }
            })
            .unwrap_or("png".to_owned());

        let filename = format!("fragment_{}.{}", fragment.id(), extension);

        let img_path = temp_dir.path().join(&filename);

        fragment.path = filename;

        image::save_buffer(
            img_path,
            &fragment.image.pixels,
            fragment.image.width,
            fragment.image.height,
            image::ColorType::Rgba8,
        )?;
    }

    let manifest_str = serde_yaml::to_string(manifest)?;

    let manifest_path = temp_dir.path().join(FILE_NAME_MANIFEST);
    write(manifest_path, manifest_str)?;

    let output = File::create(path)?;

    let mut archive = Builder::new(output);

    archive.append_dir_all(".", temp_dir.path())?;

    archive.finish()?;

    temp_dir.close()?;

    Ok(())
}
