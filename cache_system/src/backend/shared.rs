//! Backend that supports custom removal / expiry of keys
use parking_lot::Mutex;
use std::{any::Any, fmt::Debug, hash::Hash, sync::Arc};

use super::CacheBackend;

/// Cache backend that allows another backend to be shared by managing
/// a mutex internally.
///
/// This allows explicitly removing entries from the cache, for
/// example, based on a policy.
#[derive(Debug, Clone)]
pub struct SharedBackend<K, V>
where
    K: Clone + Eq + Debug + Hash + Ord + Send + 'static,
    V: Clone + Debug + Send + 'static,
{
    inner_backend: Arc<Mutex<Box<dyn CacheBackend<K = K, V = V>>>>,
}

impl<K, V> SharedBackend<K, V>
where
    K: Clone + Eq + Debug + Hash + Ord + Send + 'static,
    V: Clone + Debug + Send + 'static,
{
    /// Create new backend around the inner backend
    pub fn new(inner_backend: Box<dyn CacheBackend<K = K, V = V>>) -> Self {
        Self {
            inner_backend: Arc::new(Mutex::new(inner_backend)),
        }
    }

    /// "remove" a key (aka remove it from the shared backend) if the
    /// specified predicate is true. If the key is removed return
    /// true, otherwise return false
    ///
    /// Note that the predicate function is called while the lock is
    /// held (and thus the inner backend can't be concurrently accessed
    pub fn remove_if<P>(&self, k: &K, predicate: P) -> bool
    where
        P: Fn(V) -> bool,
    {
        let mut inner_backend = self.inner_backend.lock();
        if let Some(v) = inner_backend.get(k) {
            if predicate(v) {
                inner_backend.remove(k);
                return true;
            }
        }
        false
    }
}

impl<K, V> CacheBackend for SharedBackend<K, V>
where
    K: Clone + Eq + Debug + Hash + Ord + Send + 'static,
    V: Clone + Debug + Send + 'static,
{
    type K = K;
    type V = V;

    fn get(&mut self, k: &Self::K) -> Option<Self::V> {
        self.inner_backend.lock().get(k)
    }

    fn set(&mut self, k: Self::K, v: Self::V) {
        self.inner_backend.lock().set(k, v);
    }

    fn remove(&mut self, k: &Self::K) {
        self.inner_backend.lock().remove(k)
    }

    fn is_empty(&self) -> bool {
        self.inner_backend.lock().is_empty()
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_generic() {
        crate::backend::test_util::test_generic(|| SharedBackend::new(test_backend()))
    }

    #[test]
    fn test_is_shared() {
        let mut backend1 = SharedBackend::new(test_backend());
        let mut backend2 = backend1.clone();

        // test that a shared backend is really shared
        backend1.set(1, "foo".into());
        backend2.set(2, "bar".into());

        assert_eq!(backend1.get(&1), Some("foo".into()));
        assert_eq!(backend2.get(&1), Some("foo".into()));
        assert_eq!(backend1.get(&2), Some("bar".into()));
        assert_eq!(backend2.get(&2), Some("bar".into()));

        // make a third backend and it should also modify the previous ones
        let mut backend3 = backend1.clone();
        assert_eq!(backend3.get(&1), Some("foo".into()));
        assert_eq!(backend3.get(&2), Some("bar".into()));

        // update key 2
        backend3.set(2, "baz".into());
        assert_eq!(backend1.get(&2), Some("baz".into()));
        assert_eq!(backend2.get(&2), Some("baz".into()));
        assert_eq!(backend3.get(&2), Some("baz".into()));
    }

    #[test]
    fn test_remove_if() {
        let mut backend = SharedBackend::new(test_backend());
        backend.set(1, "foo".into());
        backend.set(2, "bar".into());

        backend.remove_if(&1, |v| v == "zzz");
        assert_eq!(backend.get(&1), Some("foo".into()));
        assert_eq!(backend.get(&2), Some("bar".into()));

        backend.remove_if(&1, |v| v == "foo");
        assert_eq!(backend.get(&1), None);
        assert_eq!(backend.get(&2), Some("bar".into()));

        backend.remove_if(&1, |v| v == "bar");
        assert_eq!(backend.get(&1), None);
        assert_eq!(backend.get(&2), Some("bar".into()));
    }

    #[test]
    fn test_remove_if_shared() {
        let mut backend = SharedBackend::new(test_backend());
        backend.set(1, "foo".into());
        backend.set(2, "bar".into());

        let backend2 = backend.clone();
        backend2.remove_if(&1, |v| v == "foo");

        // original backend should reflect the changes
        assert_eq!(backend.get(&1), None);
        assert_eq!(backend.get(&2), Some("bar".into()));
    }

    fn test_backend() -> Box<dyn CacheBackend<K = u8, V = String>> {
        Box::new(HashMap::new())
    }
}
