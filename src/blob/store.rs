use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait BlobStore: Send + Sync {
  async fn put(&self, key: &str, data: Bytes) -> anyhow::Result<()>;
  async fn get(&self, key: &str) -> anyhow::Result<Option<Bytes>>;
  async fn delete(&self, key: &str) -> anyhow::Result<()>;
}
