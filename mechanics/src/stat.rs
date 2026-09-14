use bevy::prelude::*;
use bevy_def::{
    DefComponent, DefIndex, DefParam, DefPlugin, InsertDef, RemoveDef, def_maintain_system,
};

fn notes() {
    struct StatComponent {
        current: f32,
    }

    struct StatAsset {
        default: f32,
        minimum: f32,
        maximum: f32,
    }

    struct StatScript {
        current: f32,

        default: f32,
        minimum: f32,
        maximum: f32,
    }
}

// pub type stat_maintain_system = def_maintain_system<Stat>;

pub type StatParam<'w> = DefParam<'w, Stat>;
pub type StatPlugin = DefPlugin<Stat>;
pub type StatIndex = DefIndex<Stat>;
pub type InsertStat = InsertDef<Stat>;
pub type RemoveStat = RemoveDef<Stat>;

#[derive(Reflect, Debug)]
#[repr(C)]
pub struct Stat {
    pub current: f32,
}

unsafe impl DefComponent for Stat {
    type Asset = StatAsset;

    fn defname(asset: &Self::Asset) -> std::borrow::Cow<'static, str> {
        asset.defname.clone().into()
    }
}

#[derive(Asset, Reflect, Debug)]
#[repr(C)]
pub struct StatAsset {
    pub defname: String,
    pub default: f32,
    pub minimal: f32,
    pub maximal: f32,
}
