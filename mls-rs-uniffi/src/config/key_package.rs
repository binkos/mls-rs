use mls_rs::error::IntoAnyError;
use mls_rs_core::key_package::KeyPackageData;
use mls_rs_core::mls_rs_codec::{MlsDecode, MlsEncode};
use std::fmt::Debug;

#[cfg(not(mls_build_async))]
use std::sync::Mutex;
#[cfg(mls_build_async)]
use tokio::sync::Mutex;

use crate::MlsRsError;

/// Storage for key package secrets.
///
/// Values are MLS-encoded [`KeyPackageData`] blobs.
#[cfg_attr(mls_build_async, uniffi::export(with_foreign))]
#[cfg_attr(mls_build_async, maybe_async::must_be_async)]
#[cfg_attr(not(mls_build_async), maybe_async::must_be_sync)]
#[cfg_attr(not(mls_build_async), uniffi::export(with_foreign))]
pub trait KeyPackageStorage: Send + Sync + Debug {
    async fn insert(&self, id: Vec<u8>, data: Vec<u8>) -> Result<(), MlsRsError>;
    async fn get(&self, id: Vec<u8>) -> Result<Option<Vec<u8>>, MlsRsError>;
    async fn delete(&self, id: Vec<u8>) -> Result<(), MlsRsError>;
}

/// Adapt a mls-rs `KeyPackageStorage` implementation.
///
/// This is used to adapt a mls-rs `KeyPackageStorage` implementation
/// to our own `KeyPackageStorage` trait. This way we can use any
/// standard mls-rs key package storage from the FFI layer.
#[derive(Debug)]
pub(crate) struct KeyPackageStorageAdapter<S>(Mutex<S>);

impl<S> KeyPackageStorageAdapter<S> {
    pub fn new(storage: S) -> Self {
        Self(Mutex::new(storage))
    }

    #[cfg(not(mls_build_async))]
    fn inner(&self) -> std::sync::MutexGuard<'_, S> {
        self.0.lock().unwrap()
    }

    #[cfg(mls_build_async)]
    async fn inner(&self) -> tokio::sync::MutexGuard<'_, S> {
        self.0.lock().await
    }
}

#[cfg_attr(not(mls_build_async), maybe_async::must_be_sync)]
#[cfg_attr(mls_build_async, maybe_async::must_be_async)]
impl<S, Err> KeyPackageStorage for KeyPackageStorageAdapter<S>
where
    S: mls_rs_core::key_package::KeyPackageStorage<Error = Err> + Debug,
    Err: IntoAnyError,
{
    async fn insert(&self, id: Vec<u8>, data: Vec<u8>) -> Result<(), MlsRsError> {
        let pkg = KeyPackageData::mls_decode(&mut &*data)?;
        self.inner()
            .await
            .insert(id, pkg)
            .await
            .map_err(|err| err.into_any_error().into())
    }

    async fn get(&self, id: Vec<u8>) -> Result<Option<Vec<u8>>, MlsRsError> {
        match self
            .inner()
            .await
            .get(&id)
            .await
            .map_err(|err| err.into_any_error())?
        {
            Some(pkg) => Ok(Some(pkg.mls_encode_to_vec()?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, id: Vec<u8>) -> Result<(), MlsRsError> {
        self.inner()
            .await
            .delete(&id)
            .await
            .map_err(|err| err.into_any_error().into())
    }
}
