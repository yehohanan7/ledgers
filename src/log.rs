use crate::files;
use crate::files::*;
use crate::types::Result;
use std::io::SeekFrom;
use tokio::io::AsyncWrite;

pub struct Log {
    pub id: u64,
    pub position: u64,
    file: File,
}

impl Log {
    pub async fn new(location: &PathBuf, id: u64) -> Result<Log> {
        let path = location.join(id.to_string()).with_extension("log");
        let file = files::create(&path)?;
        let position = 0;
        Ok(Log { id, file, position })
    }
    pub async fn open(location: &PathBuf, id: u64) -> Result<Log> {
        let path = location.join(id.to_string()).with_extension("log");
        let file = files::open(&path)?;
        let position = file.metadata().await.unwrap().len();
        Ok(Log { id, file, position })
    }

    pub async fn add_entry(&mut self, offset: u64, entry: Vec<u8>) -> Result<()> {
        let entry_size = entry.len() as u32;
        let total_size = 8 + 4 + entry_size as usize; // offset + entry size + entry
        let mut bytes = Vec::with_capacity(total_size);
        bytes.extend(offset.to_be_bytes().iter());
        bytes.extend(entry_size.to_be_bytes().iter());
        bytes.extend(entry.iter());
        self.file.write_all(&bytes).await?;
        self.file.flush().await?;
        self.position += total_size as u64;
        Ok(())
    }

    //TODO: use zerocopy optimisation using sendfile
    pub async fn stream_entries<T>(
        &mut self,
        position: u64,
        bytes: usize,
        target: &mut T,
    ) -> Result<()>
    where
        T: AsyncWrite + Unpin + ?Sized,
    {
        let max_bytes = self.size().await as usize;
        let bytes = if bytes > max_bytes { max_bytes } else { bytes };
        self.file.seek(SeekFrom::Start(position)).await?;
        let mut buf = vec![0; bytes];
        self.file.read_exact(&mut buf).await?;
        target.write(&buf).await?;
        self.file.seek(SeekFrom::End(0)).await?;
        Ok(())
    }

    pub async fn size(&self) -> u64 {
        self.file.metadata().await.unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util as test;

    #[tokio::test]
    async fn create_new_log() {
        let location = test::create_a_test_directory();

        let log = Log::new(&location, 123).await.unwrap();

        assert_eq!(log.position, 0);
        assert_eq!(log.size().await, 0);
    }

    #[tokio::test]
    async fn add_entry() {
        let id = 5000;
        let location = test::create_a_test_directory();
        let mut log = Log::new(&location, id).await.unwrap();

        log.add_entry(5000, vec![1, 2, 3]).await.unwrap();
        log.add_entry(5001, vec![3, 4]).await.unwrap();

        assert_eq!(log.position, 29);
    }

    #[tokio::test]
    async fn open_existing_log() {
        let location = test::create_a_test_directory();
        let id = 10;
        let mut log = Log::new(&location, id).await.unwrap();
        log.add_entry(5000, vec![1, 2, 3]).await.unwrap();

        let log = Log::open(&location, id).await.unwrap();
        assert_eq!(log.position, 15);
    }

    #[tokio::test]
    async fn stream_entries() {
        let id = 10;
        let location = test::create_a_test_directory();
        let mut log = Log::new(&location, id).await.unwrap();
        log.add_entry(10, vec![1, 2]).await.unwrap();
        log.add_entry(11, vec![3, 4]).await.unwrap();

        let mut entries = Vec::new();
        log.stream_entries(0, 16000, &mut entries).await.unwrap();

        assert_eq!(
            &entries,
            &[
                0, 0, 0, 0, 0, 0, 0, 10, //offset
                0, 0, 0, 2, //len
                1, 2, //entry
                0, 0, 0, 0, 0, 0, 0, 11, //offset
                0, 0, 0, 2, //len
                3, 4 //entry
            ]
        );
    }

    #[tokio::test]
    async fn stream_partial() {
        let id = 10;
        let location = test::create_a_test_directory();
        let mut log = Log::new(&location, id).await.unwrap();
        log.add_entry(10, vec![1, 2]).await.unwrap();
        log.add_entry(11, vec![3, 4]).await.unwrap();

        let mut entries = Vec::new();
        log.stream_entries(0, 14, &mut entries).await.unwrap();

        assert_eq!(
            &entries,
            &[
                0, 0, 0, 0, 0, 0, 0, 10, //offset
                0, 0, 0, 2, //len
                1, 2, //entry
            ]
        );
    }
}
