use std::fs;
use std::io::Result;
pub use tokio::prelude::*;

pub type File = tokio::fs::File;
pub type Path = std::path::Path;
pub type PathBuf = std::path::PathBuf;

pub fn create(path: &PathBuf) -> Result<File> {
    let dir_path = path.parent().unwrap();
    if !dir_path.exists() {
        create_dir(dir_path).unwrap();
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(path)
        .map(|f| File::from_std(f))
}

pub fn open(path: &PathBuf) -> Result<File> {
    std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map(|f| File::from_std(f))
}

pub fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
}

pub fn stem_as_u64(path: PathBuf) -> u64 {
    path.file_stem()
        .map(|name| name.to_str().unwrap())
        .unwrap()
        .to_string()
        .parse::<u64>()
        .unwrap()
}

pub fn list_files_as_u64(path: &Path, extension: &'static str) -> Result<Vec<u64>> {
    let mut entries: Vec<u64> = fs::read_dir(path)?
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().unwrap() == extension)
        .map(|p| stem_as_u64(p))
        .collect();
    entries.sort();
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use crate::files;
    use crate::files::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn create_new_file() {
        let location = Path::new("target/test_files");
        let dir_name = Uuid::new_v4().to_string();
        let path = location.join(dir_name).with_extension("txt");

        files::create(&path).unwrap();

        let file = files::open(&path).unwrap();
        assert_eq!(file.metadata().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn list_files_as_u64() {
        let location = Path::new("target/test_files").join(Uuid::new_v4().to_string());
        for i in 9..15 {
            files::create(&location.join(i.to_string()).with_extension("log")).unwrap();
            files::create(&location.join(i.to_string()).with_extension("index")).unwrap();
        }

        let dirs: Vec<u64> = files::list_files_as_u64(&location, "index").unwrap();

        assert_eq!(dirs, vec![9, 10, 11, 12, 13, 14]);
    }
}
