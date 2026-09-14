use bevy::{
    asset::{io::Reader, Asset, AssetLoader, Handle, LoadContext},
    ecs::{
        reflect::AppTypeRegistry,
        world::{FromWorld, World},
    },
    platform::collections::hash_set::HashSet,
    prelude::Component,
    reflect::{
        serde::{ReflectDeserializer, TypeRegistrationDeserializer, TypedReflectDeserializer},
        PartialReflect, TypePath, TypeRegistry, TypeRegistryArc,
    },
    tasks::ConditionalSendFuture,
};
use serde::de::{self, DeserializeSeed};
use thiserror::Error;

#[derive(Component, Clone)]
pub struct ItemHandle(pub Handle<ItemAsset>);

#[derive(Asset, TypePath, Debug)]
pub struct ItemAsset {
    pub components: Vec<Box<dyn PartialReflect>>,
}

pub struct ItemAssetLoader {
    type_registry: TypeRegistryArc,
}

impl FromWorld for ItemAssetLoader {
    fn from_world(world: &mut World) -> Self {
        Self {
            type_registry: world.resource::<AppTypeRegistry>().0.clone(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ItemAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for ItemAssetLoader {
    type Asset = ItemAsset;
    type Settings = ();
    type Error = ItemAssetLoaderError;

    fn extensions(&self) -> &[&str] {
        &["item.ron"]
    }

    fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let mut deserializer = ron::de::Deserializer::from_bytes(&bytes)?;
            let components = ComponentsDeserializer {
                registry: &self.type_registry.read(),
            };

            Ok(Self::Asset {
                components: components
                    .deserialize(&mut deserializer)
                    .map_err(|e| deserializer.span_error(e))?,
            })
        })
    }
}

struct ComponentsDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> de::DeserializeSeed<'de> for ComponentsDeserializer<'_> {
    type Value = Vec<Box<dyn PartialReflect>>;

    fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        de.deserialize_map(self)
    }
}

impl<'de> de::Visitor<'de> for ComponentsDeserializer<'_> {
    type Value = Vec<Box<dyn PartialReflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of reflect types")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut added = HashSet::new();
        let mut entries = Vec::new();
        while let Some(registration) =
            map.next_key_seed(TypeRegistrationDeserializer::new(self.registry))?
        {
            if !added.insert(registration.type_id()) {
                return Err(de::Error::custom(format_args!(
                    "duplicate reflect type: `{}`",
                    registration.type_info().type_path(),
                )));
            }

            let seed = TypedReflectDeserializer::new(registration, self.registry);
            entries.push(map.next_value_seed(seed)?);
        }
        Ok(entries)
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut dynamic_properties = Vec::new();
        while let Some(entity) = seq.next_element_seed(ReflectDeserializer::new(self.registry))? {
            dynamic_properties.push(entity);
        }
        Ok(dynamic_properties)
    }
}
