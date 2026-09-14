use super::{BoneData, ClipData, SkinData, SlotData};
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    ecs::system::Resource,
    reflect::{TypePath, TypeUuid},
    utils::BoxedFuture,
};

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct MetaData {}

#[derive(
    Asset, Default, Debug, serde::Serialize, serde::Deserialize, TypeUuid, Resource, TypePath,
)]
#[uuid = "652db5ff-ad4b-43fc-9ef8-e59a71f0eb5e"]
pub struct AnimaData {
    pub meta: MetaData,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bones: Vec<BoneData>, // parent first order

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<SlotData>, // draw order

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skins: Vec<SkinData>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clips: Vec<ClipData>,
}

#[derive(Default)]
pub struct AnimaAssetLoader;

#[allow(clippy::needless_question_mark)]
impl AssetLoader for AnimaAssetLoader {
    type Asset = AnimaData;
    type Error = ron::error::SpannedError;
    type Settings = ();

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(ron::de::from_bytes::<AnimaData>(&bytes)?)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anima"]
    }
}

/*
async fn load_data<'a, 'b>(
    bytes: &'a [u8],
    context: &'a mut LoadContext<'b>,
) -> Result<(), ron::error::SpannedError> {
    let data = ron::de::from_reader::<AnimaData>(bytes)?;
    context.set_default_asset(LoadedAsset::new(data));
    Ok(())
}

*/
