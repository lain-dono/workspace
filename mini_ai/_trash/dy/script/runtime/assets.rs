use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use std::marker::PhantomData;

/// A loader for script assets.
pub struct ScriptLoader<A: Asset + From<String>> {
    marker: PhantomData<A>,
}

impl<A: Asset + From<String>> Default for ScriptLoader<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

/// Allows providing an allow-list for extensions of AssetLoader for a Script asset
pub trait GetExtensions {
    fn extensions() -> &'static [&'static str];
}

impl<A: Asset + From<String> + GetExtensions> AssetLoader for ScriptLoader<A> {
    type Asset = A;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(A::from(String::from_utf8(bytes.to_vec())?))
    }

    fn extensions(&self) -> &[&str] {
        A::extensions()
    }
}
