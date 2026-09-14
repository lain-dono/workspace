#![warn(clippy::pedantic)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use crate::{
    agent::sync_agent,
    character::sync_character,
    coords::{CoordinateSystem, ThreeD, TwoD},
    island::sync_islands,
    nav_mesh::NavMeshAsset,
};
use bevy::app::{Plugin, RunFixedMainLoop, RunFixedMainLoopSystems};
use bevy::asset::AssetApp;
use bevy::ecs::{
    intern::Interned,
    schedule::{IntoScheduleConfigs, ScheduleLabel},
};
use bevy::prelude::*;
use std::marker::PhantomData;

pub mod disjoint_set;
pub mod half_edge;

pub mod agent;
pub mod bvh;
pub mod character;
pub mod coords;
pub mod debug;
pub mod island;
pub mod nav_mesh;
pub mod update;

pub mod landmass;
// pub mod plugin;

pub mod prelude {

    // use crate::landmass::{
    //     AgentOptions, FindPathError, FromAgentRadius, NavigationMesh, NewNodeTypeError, NodeType,
    //     PointSampleDistance3d, SamplePointError, SetNodeTypeCostError, ValidNavigationMesh,
    //     ValidationError,
    // };

    pub use crate::LandmassSystemSet;
    pub use crate::agent::{Agent2d, Agent3d, AgentState};
    pub use crate::coords::{CoordinateSystem, ThreeD, TwoD};
    pub use crate::island::{Island2d, Island3d};

    pub use crate::{LandmassPlugin2d, LandmassPlugin3d};

    pub use crate::landmass::AgentOptions;
    pub use crate::landmass::FromAgentRadius;

    // pub type Archipelago2d = crate::archipelago::Archipelago<TwoD>;
    pub type NavigationMesh2d = crate::landmass::NavMeshBuilder<TwoD>;
    pub type ValidNavigationMesh2d = crate::landmass::NavMesh<TwoD>;
    pub type NavMeshHandle2d = crate::nav_mesh::NavMeshHandle<TwoD>;
    pub type NavMesh2d = crate::nav_mesh::NavMeshAsset<TwoD>;
    pub type AgentTarget2d = crate::agent::AgentTarget<TwoD>;

    // pub type Archipelago3d = crate::archipelago::Archipelago<ThreeD>;
    pub type NavigationMesh3d = crate::landmass::NavMeshBuilder<ThreeD>;
    pub type ValidNavigationMesh3d = crate::landmass::NavMesh<ThreeD>;
    pub type NavMeshHandle3d = crate::nav_mesh::NavMeshHandle<ThreeD>;
    pub type NavMesh3d = crate::nav_mesh::NavMeshAsset<ThreeD>;
    pub type AgentTarget3d = crate::agent::AgentTarget<ThreeD>;
}

#[cfg(test)]
mod tests;

/// System set for `landmass` systems.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LandmassSystemSet {
    /// Systems for syncing the existence of components with the internal
    /// `landmass` state. Ensure your `landmass` entities are setup before this
    /// point (and not removed until [`LandmassSystemSet::Output`]).
    SyncExistence,
    /// Systems for syncing the values of components with the internal `landmass`
    /// state.
    SyncValues,
    /// The actual `landmass` updating step.
    Update,
    /// Systems for returning the output of `landmass` back to users. Avoid
    /// reading/mutating data from your `landmass` entities until after this
    /// point.
    Post,
}

pub type LandmassPlugin2d = LandmassPlugin<TwoD>;
pub type LandmassPlugin3d = LandmassPlugin<ThreeD>;

pub struct LandmassPlugin<T: CoordinateSystem> {
    pub schedule: Interned<dyn ScheduleLabel>,
    pub marker: PhantomData<T>,
}

impl<T: CoordinateSystem> Default for LandmassPlugin<T> {
    fn default() -> Self {
        Self {
            schedule: RunFixedMainLoop.intern(),
            marker: PhantomData,
        }
    }
}

impl<T: CoordinateSystem> Plugin for LandmassPlugin<T> {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_asset::<NavMeshAsset<T>>()
            .configure_sets(
                self.schedule,
                (
                    LandmassSystemSet::SyncExistence,
                    LandmassSystemSet::SyncValues,
                    LandmassSystemSet::Update,
                    LandmassSystemSet::Post,
                )
                    .chain()
                    // Configure our systems to run before physics engines.
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(
                self.schedule,
                sync_islands::<T>.in_set(LandmassSystemSet::SyncExistence),
            )
            .add_systems(
                self.schedule,
                (sync_agent::<T>, sync_character::<T>).in_set(LandmassSystemSet::SyncValues),
            )
            .add_systems(
                self.schedule,
                (
                    crate::update::navigation_update::<T>,
                    crate::update::agent_sampling::<T>,
                    crate::update::character_sampling::<T>,
                    crate::update::agent_repath::<T>,
                    crate::update::agent_movement::<T>,
                    crate::landmass::avoidance::agent_avoidance::<T>,
                )
                    .chain()
                    .in_set(LandmassSystemSet::Update),
            );
    }
}
