use crate::files;
use crate::files::*;
use crate::handle::*;
use crate::types::Result;
use std::collections::HashMap;
use tokio::io::AsyncWrite;

pub struct Segments {
    location: PathBuf,
    map: HashMap<u64, Segment>,
}

impl Segments {
    pub fn open(location: PathBuf) -> Result<Segments> {
        let ids = files::list_files_as_u64(&location, "index")?;
        let mut map = HashMap::new();
        for id in ids {
            map.insert(id, Segment::new(location.to_owned(), id));
        }
        Ok(Segments { location, map })
    }

    pub async fn create(&mut self, id: u64) -> Result<&mut Segment> {
        let segment = Segment::new(self.location.to_owned(), id);
        self.map.insert(id, segment);
        Ok(self.map.get_mut(&id).unwrap())
    }

    pub fn get(&self, id: u64) -> Option<&Segment> {
        self.map.get(&id)
    }

    pub fn create_if_absent(&mut self, id: u64) -> &mut Segment {
        if !self.map.contains_key(&id) {
            let segment = Segment::new(self.location.to_owned(), id);
            self.map.insert(id, segment);
        }
        self.map.get_mut(&id).unwrap()
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Segment> {
        self.map.get_mut(&id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

pub struct Segment {
    pub id: u64,
    location: PathBuf,
    handle: Option<Handle>,
}

impl Segment {
    pub fn new(location: PathBuf, id: u64) -> Segment {
        Segment {
            id,
            location: location,
            handle: None,
        }
    }

    pub async fn add(&mut self, entries: Vec<Vec<u8>>) -> Result<()> {
        if self.handle.is_none() {
            self.handle = Some(Handle::new(&self.location, self.id).await?)
        }
        let handle = self.handle.as_mut().unwrap();
        handle.add(entries).await?;
        Ok(())
    }

    pub async fn stream<T>(&mut self, offset: u64, bytes: usize, target: &mut T) -> Result<()>
    where
        T: AsyncWrite + Unpin + ?Sized,
    {
        if let Some(handle) = self.handle.as_mut() {
            handle.stream(offset, bytes, target).await?;
        }
        Ok(())
    }

    pub async fn size(&self) -> u64 {
        match self.handle.as_ref() {
            Some(h) => h.log_size().await,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util as test;

    #[tokio::test]
    async fn create_new_segment() {
        let location = test::create_a_test_directory();
        let mut segments = Segments::open(location).unwrap();

        let segment = segments.create(5).await.unwrap();

        assert_eq!(segment.id, 5);
    }

    #[tokio::test]
    async fn no_segments() {
        let location = test::create_a_test_directory();

        assert_eq!(Segments::open(location).unwrap().len(), 0);
    }

    #[tokio::test]
    async fn add_entries_to_segment() {
        let mut segments = Segments::open(test::create_a_test_directory()).unwrap();
        let segment = segments.create(5).await.unwrap();

        segment.add(vec![vec![1, 2], vec![5, 6]]).await.unwrap();

        assert_eq!(segment.size().await, 28);
    }

    #[tokio::test]
    async fn stream_entries_from_segment() {
        let segment_id = 5;
        let mut segments = Segments::open(test::create_a_test_directory()).unwrap();
        let segment = segments.create(segment_id).await.unwrap();
        segment.add(vec![vec![1, 2], vec![5, 6]]).await.unwrap();

        let mut buf = Vec::new();
        segment.stream(segment_id, 16000, &mut buf).await.unwrap();

        assert_eq!(
            &buf,
            &vec![
                0, 0, 0, 0, 0, 0, 0, 5, //offset
                0, 0, 0, 2, //len
                1, 2, //payload
                0, 0, 0, 0, 0, 0, 0, 6, //offset
                0, 0, 0, 2, //len
                5, 6 //payload
            ]
        );
    }
}
