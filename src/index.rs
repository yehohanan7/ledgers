use crate::files;
use crate::files::*;
use crate::types::*;
use byteorder::{BigEndian, ByteOrder};
use std::io::SeekFrom;

const ENTRY_SIZE: usize = 8;

pub struct Index {
    pub id: u64,
    pub base_offset: u64,
    pub next_offset: u64,
    file: File,
}

impl Index {
    pub async fn new(location: &PathBuf, id: u64) -> Result<Index> {
        let path = location.join(id.to_string()).with_extension("index");
        let file = files::create(&path)?;
        let (base_offset, next_offset) = (id, id);
        Ok(Index {
            id,
            base_offset,
            next_offset,
            file,
        })
    }

    pub async fn open(location: &PathBuf, id: u64) -> Result<Index> {
        let path = location.join(id.to_string()).with_extension("index");
        let file = files::open(&path)?;
        let size = file.metadata().await.unwrap().len();
        let entries = size / ENTRY_SIZE as u64;
        let base_offset = id;
        let next_offset = base_offset + entries;
        Ok(Index {
            id,
            base_offset,
            next_offset,
            file,
        })
    }

    pub async fn add_entry(&mut self, value: u64) -> Result<()> {
        let mut buf: [u8; ENTRY_SIZE] = [0; ENTRY_SIZE];
        BigEndian::write_u64(&mut buf, value);
        self.file.write_all(&buf).await?;
        self.file.flush().await?; //TODO: Flush from segment as a batch
        self.next_offset += 1;
        Ok(())
    }

    pub async fn find_entry(&mut self, offset: u64) -> Result<u64> {
        let index_offset = (offset - self.base_offset) * ENTRY_SIZE as u64;
        self.file.seek(SeekFrom::Start(index_offset)).await?;
        let mut buf: [u8; ENTRY_SIZE] = [0; ENTRY_SIZE];
        self.file.read_exact(&mut buf).await?;
        self.file.seek(SeekFrom::End(0)).await?;
        Ok(BigEndian::read_u64(&buf))
    }

    pub async fn size(&self) -> u64 {
        self.file.metadata().await.unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util as test;
    use uuid::Uuid;

    #[tokio::test]
    async fn create_new_index() {
        let location = test::create_a_test_directory();
        let base_offset = 5000;

        let index = Index::new(&location.to_owned(), base_offset).await.unwrap();

        assert_eq!(index.next_offset, base_offset);
    }

    #[tokio::test]
    async fn next_offset_for_existing_index() {
        let location = test::create_a_test_directory();
        files::create(&location.join("1000").with_extension("index")).unwrap();

        let mut index = Index::open(&location.to_owned(), 1000).await.unwrap();
        index.add_entry(100).await.unwrap();
        index.add_entry(101).await.unwrap();

        assert_eq!(index.next_offset, 1002);
    }

    #[tokio::test]
    async fn find_entry() {
        let base_offset = 5000;
        let location = test::create_a_test_directory();
        let mut index = Index::new(&location.to_owned(), base_offset).await.unwrap();
        index.add_entry(100).await.unwrap();
        index.add_entry(101).await.unwrap();
        index.add_entry(102).await.unwrap();

        assert_eq!(index.find_entry(5001).await.unwrap(), 101);

        index.add_entry(103).await.unwrap();
        assert_eq!(index.find_entry(5002).await.unwrap(), 102);
        assert_eq!(index.find_entry(5003).await.unwrap(), 103);
    }
}
