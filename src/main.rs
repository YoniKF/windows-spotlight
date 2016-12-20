extern crate crypto;
extern crate image;

use std::env;
use std::error;
use std::io::{self, Read};
use std::fmt;
use std::fs::{self, File, ReadDir};
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

#[derive(Debug)]
enum AssetsReadingError {
    EnvVar(env::VarError),
    Io(io::Error),
}

impl fmt::Display for AssetsReadingError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AssetsReadingError::EnvVar(ref error) => {
                write!(formatter, "Environment variable error: {}", error)
            }
            AssetsReadingError::Io(ref error) => write!(formatter, "IO error: {}", error),
        }
    }
}

impl error::Error for AssetsReadingError {
    fn description(&self) -> &str {
        match *self {
            AssetsReadingError::EnvVar(ref error) => error.description(),
            AssetsReadingError::Io(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            AssetsReadingError::EnvVar(ref error) => Some(error),
            AssetsReadingError::Io(ref error) => Some(error),
        }
    }
}

impl From<env::VarError> for AssetsReadingError {
    fn from(error: env::VarError) -> AssetsReadingError {
        AssetsReadingError::EnvVar(error)
    }
}

impl From<io::Error> for AssetsReadingError {
    fn from(error: io::Error) -> AssetsReadingError {
        AssetsReadingError::Io(error)
    }
}

fn read_assets_directory() -> Result<ReadDir, AssetsReadingError> {
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

fn main() {
    let user_profile_env_var = env::var(USER_PROFILE_ENV_VAR).unwrap();
    let user_profile = Path::new(&user_profile_env_var);
    let wallpapers = user_profile.join(WALLPAPERS_RELATIVE_PATH);

    match read_assets_directory() {
        Ok(assets) => {
            for asset in assets {
                let asset = asset.unwrap();
                if asset.file_type().unwrap().is_file() {
                    if let Ok(dimensions) = JPEGDecoder::new(File::open(asset.path()).unwrap())
                        .dimensions() {
                        if is_full_hd_or_better(dimensions) {
                            let path = asset.path();
                            let mut new_path =
                                wallpapers.join(calculate_file_md5_digest(&path).unwrap());
                            new_path.set_extension(JPEG_EXTENSION);
                            if !new_path.exists() {
                                fs::copy(path, new_path).unwrap();
                            }
                        }
                    }
                }
            }
        }
        Err(error) => println!("{}", error),
    }
}