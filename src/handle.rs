use crate::files;
use crate::files::*;
use crate::index::*;
use crate::log::*;
use crate::types::Result;
use tokio::io::AsyncWrite;

pub struct Handle {
    log: Log,
    index: Index,
}

impl Handle {
    pub async fn new(location: &PathBuf, id: u64) -> Result<Handle> {
        let log = Log::new(location, id).await?;
        let index = Index::new(location, id).await?;
        Ok(Handle { log, index })
    }

    pub async fn open(location: &PathBuf, id: u64) -> Result<Handle> {
        let log = Log::open(location, id).await?;
        let index = Index::open(location, id).await?;
        Ok(Handle { log, index })
    }

    pub async fn stream<T>(&mut self, offset: u64, bytes: usize, target: &mut T) -> Result<()>
    where
        T: AsyncWrite + Unpin + ?Sized,
    {
        let position = self.index.find_entry(offset).await?;
        self.log.stream_entries(position, bytes, target).await?;
        Ok(())
    }

    pub async fn add(&mut self, entries: Vec<Vec<u8>>) -> Result<()> {
        for entry in entries {
            let log_position = self.log.position;
            let offset = self.index.next_offset;
            self.log.add_entry(offset, entry).await?;
            self.index.add_entry(log_position).await?;
        }
        Ok(())
    }

    pub async fn log_size(&self) -> u64 {
        self.log.size().await
    }

    pub async fn index_size(&self) -> u64 {
        self.index.size().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util as test;

    #[tokio::test]
    async fn create_new_handle() {
        let id = 123;
        let location = test::create_a_test_directory();

        Handle::new(&location, id).await.unwrap();

        assert!(location
            .join(id.to_string())
            .with_extension("index")
            .exists());
        assert!(location.join(id.to_string()).with_extension("log").exists());
    }

    #[tokio::test]
    async fn open_existing_handle() {
        let id = 123;
        let location = test::create_a_test_directory();
        let mut handle = Handle::new(&location, id).await.unwrap();
        handle.add(vec![vec![1, 2], vec![5, 6]]).await.unwrap();

        let handle = Handle::open(&location, id).await.unwrap();

        assert_eq!(handle.log_size().await, 28);
        assert_eq!(handle.index_size().await, 16);
    }

    #[tokio::test]
    async fn read_entries_from_active_handle() {
        let id = 123;
        let location = test::create_a_test_directory();
        let mut handle = Handle::new(&location, id).await.unwrap();
        handle.add(vec![vec![1, 2], vec![3, 4]]).await.unwrap();

        let mut buf: Vec<u8> = Vec::new();

        handle.stream(123, 16000, &mut buf).await.unwrap();
        assert_eq!(
            &buf,
            &[
                0, 0, 0, 0, 0, 0, 0, 123, //offset
                0, 0, 0, 2, //len
                1, 2, //entry
                0, 0, 0, 0, 0, 0, 0, 124, //offset
                0, 0, 0, 2, //entry
                3, 4 //payload
            ]
        );
    }
}
