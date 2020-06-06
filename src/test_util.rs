use crate::files;
use crate::files::*;
use uuid::Uuid;

pub fn create_a_test_directory() -> PathBuf {
    let location = Path::new("target/my_ledgers")
        .join(Uuid::new_v4().to_string())
        .to_owned();
    files::create_dir(&location).unwrap();
    location
}
