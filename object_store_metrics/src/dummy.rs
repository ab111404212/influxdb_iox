//! Crate that mimics the interface of the the various object stores
//! but does nothing if they are not enabled.

use async_trait::async_trait;
use bytes::Bytes;
use snafu::Snafu;

use object_store::{path::Path, GetResult, ListResult, ObjectMeta, ObjectStore, Result};

/// A specialized `Error` for Azure object store-related errors
#[derive(Debug, Snafu, Clone)]
#[allow(missing_copy_implementations, missing_docs)]
enum Error {
    #[snafu(display(
        "'{}' not supported with this build. Hint: recompile with appropriate features",
        name
    ))]
    NotSupported { name: &'static str },
}

impl From<Error> for object_store::Error {
    fn from(source: Error) -> Self {
        match source {
            Error::NotSupported { name } => Self::Generic {
                store: name,
                source: Box::new(source),
            },
        }
    }
}

#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
/// An object store that always generates an error
pub struct DummyObjectStore {
    name: &'static str,
}

impl DummyObjectStore {
    /// Create a new [`DummyObjectStore`] that always fails
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl std::fmt::Display for DummyObjectStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dummy({})", self.name)
    }
}

#[async_trait]
impl ObjectStore for DummyObjectStore {
    async fn put(&self, _location: &Path, _bytes: Bytes) -> Result<()> {
        Ok(NotSupportedSnafu { name: self.name }.fail()?)
    }

    async fn get(&self, _location: &Path) -> Result<GetResult> {
        Ok(NotSupportedSnafu { name: self.name }.fail()?)
    }

    async fn head(&self, _location: &Path) -> Result<ObjectMeta> {
        Ok(NotSupportedSnafu { name: self.name }.fail()?)
    }

    async fn delete(&self, _location: &Path) -> Result<()> {
        Ok(NotSupportedSnafu { name: self.name }.fail()?)
    }

    async fn list<'a>(
        &'a self,
        _prefix: Option<&'a Path>,
    ) -> Result<futures::stream::BoxStream<'a, Result<ObjectMeta>>> {
        Ok(NotSupportedSnafu { name: self.name }.fail()?)
    }

    async fn list_with_delimiter(&self, _prefix: &Path) -> Result<ListResult> {
        Ok(NotSupportedSnafu { name: self.name }.fail()?)
    }
}
