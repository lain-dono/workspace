use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    reflect::TypePath,
};
use thiserror::Error;

pub fn plugin(app: &mut App) {
    app.init_asset::<MotiveAsset>()
        .init_asset_loader::<MotiveAssetLoader>()
        .init_resource::<State>()
        .add_systems(Startup, setup)
        .add_systems(Update, print_on_load);
}

#[derive(Resource, Default)]
struct State {
    handle: Handle<MotiveAsset>,
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    state.handle = asset_server.load("ai/motives.ron");
}

fn print_on_load(
    mut printed: Local<bool>,
    state: Res<State>,
    motive_assets: Res<Assets<MotiveAsset>>,
) {
    let motive_asset = motive_assets.get(&state.handle);
    if *printed {
        return;
    }

    if motive_asset.is_none() {
        info!("Motive Asset Not Ready");
        return;
    }

    info!("Motive asset loaded: {:?}", motive_asset.unwrap());

    *printed = true;
}

pub type MotiveCurve = ai::Curve<6>;

#[derive(Asset, TypePath, Debug, serde::Serialize, serde::Deserialize)]
pub struct MotiveAsset(pub HashMap<String, MotiveCurve>);

#[derive(Default)]
pub struct MotiveAssetLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum MotiveAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),

    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for MotiveAssetLoader {
    type Asset = MotiveAsset;
    type Settings = ();
    type Error = MotiveAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes::<MotiveAsset>(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
