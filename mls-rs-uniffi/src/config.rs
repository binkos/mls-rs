use std::sync::Arc;

use mls_rs::{
    client_builder::{self, WithGroupStateStorage, WithKeyPackageRepo},
    identity::basic,
    storage_provider::in_memory::{InMemoryGroupStateStorage, InMemoryKeyPackageStorage},
};
use mls_rs_core::key_package::KeyPackageData;
use mls_rs_core::mls_rs_codec::{MlsDecode, MlsEncode};
use mls_rs_crypto_rustcrypto::RustCryptoProvider;
use zeroize::Zeroizing;

use self::group_state::{GroupStateStorage, GroupStateStorageAdapter};
use self::key_package::{KeyPackageStorage, KeyPackageStorageAdapter};
use crate::Error;

pub mod group_state;
pub mod key_package;

#[derive(Debug, Clone)]
pub(crate) struct ClientGroupStorage(Arc<dyn GroupStateStorage>);

impl From<Arc<dyn GroupStateStorage>> for ClientGroupStorage {
    fn from(value: Arc<dyn GroupStateStorage>) -> Self {
        Self(value)
    }
}

#[cfg_attr(not(mls_build_async), maybe_async::must_be_sync)]
#[cfg_attr(mls_build_async, maybe_async::must_be_async)]
impl mls_rs_core::group::GroupStateStorage for ClientGroupStorage {
    type Error = Error;

    async fn state(&self, group_id: &[u8]) -> Result<Option<Zeroizing<Vec<u8>>>, Self::Error> {
        let data = self.0.state(group_id.to_vec()).await?;
        Ok(data.map(Into::into))
    }

    async fn epoch(
        &self,
        group_id: &[u8],
        epoch_id: u64,
    ) -> Result<Option<Zeroizing<Vec<u8>>>, Self::Error> {
        let data = self.0.epoch(group_id.to_vec(), epoch_id).await?;
        Ok(data.map(Into::into))
    }

    async fn write(
        &mut self,
        state: mls_rs_core::group::GroupState,
        inserts: Vec<mls_rs_core::group::EpochRecord>,
        updates: Vec<mls_rs_core::group::EpochRecord>,
    ) -> Result<(), Self::Error> {
        self.0
            .write(
                state.id,
                state.data.to_vec(),
                inserts.into_iter().map(Into::into).collect(),
                updates.into_iter().map(Into::into).collect(),
            )
            .await
    }

    async fn max_epoch_id(&self, group_id: &[u8]) -> Result<Option<u64>, Self::Error> {
        self.0.max_epoch_id(group_id.to_vec()).await
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ClientKeyPackageStorage(Arc<dyn KeyPackageStorage>);

impl From<Arc<dyn KeyPackageStorage>> for ClientKeyPackageStorage {
    fn from(value: Arc<dyn KeyPackageStorage>) -> Self {
        Self(value)
    }
}

#[cfg_attr(not(mls_build_async), maybe_async::must_be_sync)]
#[cfg_attr(mls_build_async, maybe_async::must_be_async)]
impl mls_rs_core::key_package::KeyPackageStorage for ClientKeyPackageStorage {
    type Error = Error;

    async fn insert(&mut self, id: Vec<u8>, pkg: KeyPackageData) -> Result<(), Self::Error> {
        let data = pkg.mls_encode_to_vec()?;
        self.0.insert(id, data).await
    }

    async fn get(&self, id: &[u8]) -> Result<Option<KeyPackageData>, Self::Error> {
        match self.0.get(id.to_vec()).await? {
            Some(data) => {
                let pkg = KeyPackageData::mls_decode(&mut &*data)?;
                Ok(Some(pkg))
            }
            None => Ok(None),
        }
    }

    async fn delete(&mut self, id: &[u8]) -> Result<(), Self::Error> {
        self.0.delete(id.to_vec()).await
    }
}

pub type UniFFIConfig = client_builder::WithIdentityProvider<
    basic::BasicIdentityProvider,
    client_builder::WithCryptoProvider<
        RustCryptoProvider,
        WithGroupStateStorage<
            ClientGroupStorage,
            WithKeyPackageRepo<ClientKeyPackageStorage, client_builder::BaseConfig>,
        >,
    >,
>;

/// Client configuration using callback-based storage.
///
/// Group state is provided via the `group_state_storage` callback.
/// Key packages are provided via the `key_package_storage` callback.
#[derive(Debug, Clone, uniffi::Record)]
pub struct ClientConfig {
    pub group_state_storage: Arc<dyn GroupStateStorage>,
    pub key_package_storage: Arc<dyn KeyPackageStorage>,
    /// Use the ratchet tree extension. If this is false, then you
    /// must supply `ratchet_tree` out of band to clients.
    pub use_ratchet_tree_extension: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            group_state_storage: Arc::new(GroupStateStorageAdapter::new(
                InMemoryGroupStateStorage::new(),
            )),
            key_package_storage: Arc::new(KeyPackageStorageAdapter::new(
                InMemoryKeyPackageStorage::default(),
            )),
            use_ratchet_tree_extension: true,
        }
    }
}

// TODO(mgeisler): turn into an associated function when UniFFI
// supports them: https://github.com/mozilla/uniffi-rs/issues/1074.
/// Create a client config with an in-memory group state storage.
#[uniffi::export]
pub fn client_config_default() -> ClientConfig {
    ClientConfig::default()
}
