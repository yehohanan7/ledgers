use crate::files::*;
use crate::segment::*;
use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWrite;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct LedgerRepository {
    ledgers: Arc<RwLock<HashMap<String, Ledger>>>,
}

impl LedgerRepository {
    pub fn new() -> LedgerRepository {
        LedgerRepository {
            ledgers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, location: &Path, segment_size: u64) -> Result<String> {
        let ledgers = self.ledgers.clone();
        let mut ledgers = ledgers.write().await;
        let ledger = Ledger::new(location, segment_size).await?;
        let id = ledger.id.clone();
        ledgers.insert(id.clone(), ledger);
        Ok(id)
    }
}

pub fn new_repository() -> LedgerRepository {
    LedgerRepository::new()
}

pub struct Ledger {
    pub id: String,
    segment_size: u64,
    segments: Segments,
}

impl Ledger {
    pub async fn new(location: &Path, segment_size: u64) -> Result<Ledger> {
        let id = Uuid::new_v4().to_string();
        let path = PathBuf::from(location).join(&id);
        create_dir(&path)?;
        Ok(Ledger {
            id,
            segment_size,
            segments: Segments::open(path)?,
        })
    }

    pub async fn open(location: &Path, id: String, segment_size: u64) -> Result<Option<Ledger>> {
        let path: PathBuf = location.join(&id);
        if !path.exists() {
            Ok(None)
        } else {
            Ok(Some(Ledger {
                id,
                segment_size,
                segments: Segments::open(path)?,
            }))
        }
    }

    pub async fn add(&mut self, segment_id: u64, entries: Vec<Vec<u8>>) -> Result<()> {
        let segment = self.segments.create_if_absent(segment_id);
        if segment.size().await >= self.segment_size {
            Err(Error::SegmentFull(segment_id))
        } else {
            segment.add(entries).await?;
            Ok(())
        }
    }

    pub async fn stream<T>(
        &mut self,
        segment_id: u64,
        offset: u64,
        bytes: usize,
        target: &mut T,
    ) -> Result<()>
    where
        T: AsyncWrite + Unpin + ?Sized,
    {
        if let Some(segment) = self.segments.get_mut(segment_id) {
            segment.stream(offset, bytes, target).await?;
        }
        Ok(())
    }

    pub async fn segment_size(&self, segment_id: u64) -> u64 {
        match self.segments.get(segment_id) {
            Some(segment) => segment.size().await,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util as test;

    #[tokio::test]
    async fn create_new_ledger() {
        let location = test::create_a_test_directory();

        let ledger = Ledger::new(&location, 100).await.unwrap();

        assert_eq!(ledger.id.len(), 36);
        assert!(location.join(ledger.id).exists());
    }

    #[tokio::test]
    async fn indicate_missing_ledger() {
        let location = test::create_a_test_directory();
        let id = "unknown_id".to_owned();

        let ledger = Ledger::open(&location, id, 100).await.unwrap();

        assert!(ledger.is_none());
    }

    #[tokio::test]
    async fn add_entries_to_ledger() {
        let segment_id = 10;
        let location = test::create_a_test_directory();
        let entries = vec![vec![1, 2], vec![3, 4]];
        let mut ledger = Ledger::new(&location, 100).await.unwrap();

        ledger.add(segment_id, entries).await.unwrap();

        assert_eq!(ledger.segment_size(segment_id).await, 28);
    }

    #[tokio::test]
    async fn segment_full() {
        let segment_id = 10;
        let location = test::create_a_test_directory();
        let entries = vec![vec![1, 2], vec![3, 4]];
        let mut ledger = Ledger::new(&location, 28).await.unwrap();
        ledger.add(segment_id, entries).await.unwrap();

        let result = ledger.add(segment_id, vec![vec![1]]).await;

        assert!(result.err().unwrap().is_segment_full());
        let offset = 10;
        let bytes = 16000;
        let mut buf = Vec::new();
        ledger
            .stream(segment_id, offset, bytes, &mut buf)
            .await
            .unwrap();
        assert_eq!(
            &buf,
            &vec![
                0, 0, 0, 0, 0, 0, 0, 10, //offset
                0, 0, 0, 2, //size
                1, 2, //entry
                0, 0, 0, 0, 0, 0, 0, 11, //offset
                0, 0, 0, 2, //size
                3, 4 //entry
            ]
        );
    }

    #[tokio::test]
    async fn stream_entries_from_ledger() {
        let location = test::create_a_test_directory();
        let segment_id = 10;
        let entries = vec![vec![1, 2], vec![3, 4]];
        let mut ledger = Ledger::new(&location, 100).await.unwrap();
        ledger.add(segment_id, entries).await.unwrap();

        let mut buf = Vec::new();
        let offset = 10;
        let bytes = 16000;
        ledger
            .stream(segment_id, offset, bytes, &mut buf)
            .await
            .unwrap();

        assert_eq!(
            &buf,
            &vec![
                0, 0, 0, 0, 0, 0, 0, 10, //offset
                0, 0, 0, 2, //size
                1, 2, //entry
                0, 0, 0, 0, 0, 0, 0, 11, //offset
                0, 0, 0, 2, //size
                3, 4 //entry
            ]
        );
    }
}
