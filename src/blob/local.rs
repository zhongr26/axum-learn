use std::path::PathBuf;

use async_trait::async_trait;
use bytes::Bytes;
use tokio::fs;

use crate::blob::store::BlobStore;

pub struct LocalBlobStore {
  pub root: PathBuf,
}

#[async_trait]
impl BlobStore for LocalBlobStore {
  async fn put(&self, key: &str, data: Bytes) -> anyhow::Result<()> {
    let path = self.root.join(key);
    // 生产模式需防路径穿透：key 只允许 [a-zA-Z0-9-_/.]
    fs::create_dir_all(path.parent().unwrap()).await?;
    fs::write(path, data).await?;
    Ok(())
  }

  async fn get(&self, key: &str) -> anyhow::Result<Option<Bytes>> {
    match fs::read(self.root.join(key)).await {
      Ok(b) => Ok(Some(b.into())),
      Err(e) if e.kind() == tokio::io::ErrorKind::NotFound => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  async fn delete(&self, key: &str) -> anyhow::Result<()> {
    match fs::remove_file(self.root.join(key)).await {
      Ok(()) => Ok(()),
      Err(e) if e.kind() == tokio::io::ErrorKind::NotFound => Ok(()),
      Err(e) => Err(e.into()),
    }
  }
}
