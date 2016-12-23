extern crate crypto;
extern crate docopt;
extern crate image;
extern crate rustc_serialize;

mod errors;

use std::env;
use std::io::{self, Read};
use std::fs::{self, DirEntry, File, ReadDir};
use std::path::Path;

use crypto::digest::Digest;
use crypto::md5::Md5;

use docopt::Docopt;

use image::ImageDecoder;
use image::jpeg::JPEGDecoder;

use errors::WindowsSpotlightResult;

const USAGE: &'static str = "
Windows Spotlight Collector.

Usage:
  windows_spotlight.exe <destination>
  windows_spotlight.exe (-h | --help)

Options:
-h --help       Show this screen.
";

const USER_PROFILE_ENV_VAR: &'static str = "UserProfile";
const ASSETS_RELATIVE_PATH: &'static str = r#"AppData\Local\Packages\Microsoft.Windows.ContentDeliveryManager_cw5n1h2txyewy\LocalState\Assets"#;
const JPEG_EXTENSION: &'static str = "jpg";

const FHD: (u32, u32) = (1920, 1080);

#[derive(RustcDecodable)]
struct Arguments {
    arg_destination: String,
}

fn read_assets_directory() -> WindowsSpotlightResult<ReadDir> {
    Ok(Path::new(&env::var(USER_PROFILE_ENV_VAR)?).join(ASSETS_RELATIVE_PATH)
        .read_dir()?)
}

fn is_full_hd_or_better(dimensions: (u32, u32)) -> bool {
    FHD <= dimensions && dimensions.0 * FHD.1 == dimensions.1 * FHD.0
}

fn should_collect(asset: &DirEntry) -> io::Result<bool> {
    if asset.file_type()?.is_file() {
        if let Ok(dimensions) = JPEGDecoder::new(File::open(asset.path())?).dimensions() {
            if is_full_hd_or_better(dimensions) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    File::open(path)?.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn calculate_file_md5_digest(path: &Path) -> io::Result<String> {
    let mut md5 = Md5::new();
    md5.input(read_file(path)?.as_slice());
    Ok(md5.result_str())
}

fn collect_image(from: &Path, to: &Path) -> io::Result<()> {
    let mut to = to.join(calculate_file_md5_digest(from)?);
    to.set_extension(JPEG_EXTENSION);
    if !to.exists() {
        fs::copy(from, to)?;
    }
    Ok(())
}

fn process_assets(destination: &Path) -> WindowsSpotlightResult<()> {
    for asset in read_assets_directory()? {
        let asset = asset?;
        if should_collect(&asset)? {
            collect_image(&asset.path(), destination)?;
        }
    }
    Ok(())
}

fn main() {
    let arguments: Arguments = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    process_assets(Path::new(&arguments.arg_destination)).unwrap_or_else(|e| e.exit());
}