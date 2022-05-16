#![deny(rustdoc::broken_intra_doc_links, rustdoc::bare_urls, rust_2018_idioms)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::explicit_iter_loop,
    clippy::future_not_send,
    clippy::use_self,
    clippy::clone_on_ref_ptr
)]

pub mod chunk;
pub mod metadata;
pub mod storage;

use data_types::{NamespaceId, PartitionId, ShardId, TableId};
use object_store::path::Path;
use uuid::Uuid;

/// Location of a Parquet file within a database's object store.
/// The exact format is an implementation detail and is subject to change.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ParquetFilePath {
    namespace_id: NamespaceId,
    table_id: TableId,
    shard_id: ShardId,
    partition_id: PartitionId,
    object_store_id: Uuid,
}

impl ParquetFilePath {
    /// Create parquet file path relevant for the storage layout.
    pub fn new(
        namespace_id: NamespaceId,
        table_id: TableId,
        shard_id: ShardId,
        partition_id: PartitionId,
        object_store_id: Uuid,
    ) -> Self {
        Self {
            namespace_id,
            table_id,
            shard_id,
            partition_id,
            object_store_id,
        }
    }

    /// Get object-store path.
    pub fn object_store_path(&self) -> Path {
        let Self {
            namespace_id,
            table_id,
            shard_id,
            partition_id,
            object_store_id,
        } = self;

        Path::from_iter([
            namespace_id.to_string().as_str(),
            table_id.to_string().as_str(),
            shard_id.to_string().as_str(),
            partition_id.to_string().as_str(),
            &format!("{}.parquet", object_store_id),
        ])
    }
}

impl From<&Self> for ParquetFilePath {
    fn from(borrowed: &Self) -> Self {
        *borrowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parquet_file_absolute_dirs_and_file_path() {
        let pfp = ParquetFilePath::new(
            NamespaceId::new(1),
            TableId::new(2),
            ShardId::new(3),
            PartitionId::new(4),
            Uuid::nil(),
        );
        let path = pfp.object_store_path();
        assert_eq!(
            path.to_string(),
            "1/2/3/4/00000000-0000-0000-0000-000000000000.parquet".to_string(),
        );
    }
}
