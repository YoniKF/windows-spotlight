extern crate crypto;
extern crate image;

mod errors;

use std::env;
use std::io::{self, Read};
use std::fs::{self, File, ReadDir};
use std::path::Path;

use crypto::digest::Digest;
use crypto::md5::Md5;

use image::ImageDecoder;
use image::jpeg::JPEGDecoder;

use errors::WindowsSpotlightResult;

static USER_PROFILE_ENV_VAR: &'static str = "UserProfile";
static ASSETS_RELATIVE_PATH: &'static str = r#"AppData\Local\Packages\Microsoft.Windows.ContentDeliveryManager_cw5n1h2txyewy\LocalState\Assets"#;
static WALLPAPERS_RELATIVE_PATH: &'static str = r#"Pictures\Wallpapers\Windows Spotlight 2"#;
static JPEG_EXTENSION: &'static str = "jpg";

const FHD: (u32, u32) = (1920, 1080);

fn read_assets_directory() -> WindowsSpotlightResult<ReadDir> {
    Ok(Path::new(&env::var(USER_PROFILE_ENV_VAR)?).join(ASSETS_RELATIVE_PATH)
        .read_dir()?)
}

fn is_full_hd_or_better(dimensions: (u32, u32)) -> bool {
    FHD <= dimensions && dimensions.0 * FHD.1 == dimensions.1 * FHD.0
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

fn process_assets(destination: &Path) -> WindowsSpotlightResult<()> {
    for asset in read_assets_directory()? {
        let asset = asset?;
        if asset.file_type()?.is_file() {
            if let Ok(dimensions) = JPEGDecoder::new(File::open(asset.path())?).dimensions() {
                if is_full_hd_or_better(dimensions) {
                    let path = asset.path();
                    let mut new_path = destination.join(calculate_file_md5_digest(&path)?);
                    new_path.set_extension(JPEG_EXTENSION);
                    if !new_path.exists() {
                        fs::copy(path, new_path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let user_profile_env_var = env::var(USER_PROFILE_ENV_VAR).unwrap();
    let user_profile = Path::new(&user_profile_env_var);
    let wallpapers = user_profile.join(WALLPAPERS_RELATIVE_PATH);

    if let Err(error) = process_assets(&wallpapers) {
        println!("{}", error)
    }
}