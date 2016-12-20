extern crate crypto;
extern crate image;

use std::env;
use std::io::Read;
use std::fs;
use std::fs::File;
use std::path::Path;

use crypto::digest::Digest;
use crypto::md5::Md5;

use image::ImageDecoder;
use image::jpeg::JPEGDecoder;

static USER_PROFILE_ENV_VAR: &'static str = "UserProfile";
static ASSETS_RELATIVE_PATH: &'static str = r#"AppData\Local\Packages\Microsoft.Windows.ContentDeliveryManager_cw5n1h2txyewy\LocalState\Assets"#;
static WALLPAPERS_RELATIVE_PATH: &'static str = r#"Pictures\Wallpapers\Windows Spotlight 2"#;
static JPEG_EXTENSION: &'static str = "jpg";

const FHD: (u32, u32) = (1920, 1080);

fn is_full_hd_or_better(dimensions: (u32, u32)) -> bool {
    FHD <= dimensions && dimensions.0 * FHD.1 == dimensions.1 * FHD.0
}

fn read_file(path: &Path) -> Vec<u8> {
    let mut buffer = Vec::new();
    File::open(path).and_then(|mut file| file.read_to_end(&mut buffer));
    buffer
}

fn calculate_file_md5_digest(path: &Path) -> String {
    let buffer = read_file(path);
    let mut md5 = Md5::new();
    md5.input(buffer.as_slice());
    md5.result_str()
}

fn main() {
    let user_profile_env_var = env::var(USER_PROFILE_ENV_VAR).unwrap();
    let user_profile = Path::new(&user_profile_env_var);
    let assets = user_profile.join(ASSETS_RELATIVE_PATH);
    let wallpapers = user_profile.join(WALLPAPERS_RELATIVE_PATH);

    for asset in assets.read_dir().unwrap() {
        let asset = asset.unwrap();
        if asset.file_type().unwrap().is_file() {
            if let Ok(dimensions) = JPEGDecoder::new(File::open(asset.path()).unwrap())
                .dimensions() {
                if is_full_hd_or_better(dimensions) {
                    let path = asset.path();
                    let mut new_path = wallpapers.join(calculate_file_md5_digest(&path));
                    new_path.set_extension(JPEG_EXTENSION);
                    if !new_path.exists() {
                        fs::copy(path, new_path).unwrap();
                    }
                }
            }
        }
    }
}