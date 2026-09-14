use std::time::Duration;

use crate::debug::DebugDrawError;
use crate::landmass::debug::{DebugDrawer, draw_archipelago_debug};
use crate::landmass::pathfinding::PathResult;
use crate::landmass::{
    Agent, AgentId, AgentOptions, Archipelago, Character, CharacterId, CoordinateSystem, Island,
    IslandId, SamplePointError, SampledPoint, nav_data::NodeRef,
};
use crate::landmass::{FindPathError, NodeType};
use bevy::ecs::schedule::{IntoScheduleConfigs, Schedule, SystemSet};
use bevy::ecs::system::{Res, ResMut};
use bevy::platform::collections::HashMap;
use bevy::time::Time;
use bevy::{
    ecs::{
        entity::Entity,
        query::QueryState,
        system::{Query, SystemState},
        world::{Mut, World},
    },
    math::Vec3,
};

mod agent;
mod astar;
mod avoidance;
mod bvh;
mod debug;
mod geometry;
mod main;
mod nav_data;
mod nav_mesh;
mod path;
mod pathfinding;
mod query;

mod plugin_debug;
// mod plugin_mod;
mod nav_mesh_convert;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlushEvents;

pub struct ArchipelagoWorld<T: CoordinateSystem> {
    world: World,
    schedule: Schedule,

    avoidance_schedule: Schedule,

    agents: QueryState<(Entity, &'static mut Agent<T>)>,
    islands: QueryState<(Entity, &'static mut Island<T>)>,
    characters: QueryState<(Entity, &'static mut Character<T>)>,

    marker: std::marker::PhantomData<T>,
}

impl<T: CoordinateSystem> ArchipelagoWorld<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::with_config(0.01)
    }

    #[allow(clippy::new_without_default)]
    pub fn with_config(edge_link_distance: f32) -> Self {
        let mut world = World::new();

        world.register_component::<Island<T>>();
        world.register_component::<Agent<T>>();
        world.register_component::<Character<T>>();

        world.init_resource::<Time>();
        world.insert_resource(Archipelago::<T>::new(edge_link_distance));

        let mut schedule = Schedule::default();

        schedule.add_systems(
            (
                crate::update::navigation_update::<T>,
                crate::update::agent_sampling::<T>,
                crate::update::character_sampling::<T>,
                crate::update::agent_repath::<T>,
                crate::update::agent_movement::<T>,
                crate::landmass::avoidance::agent_avoidance::<T>,
            )
                .chain(),
        );

        let mut avoidance_schedule = Schedule::default();
        avoidance_schedule.add_systems(crate::landmass::avoidance::agent_avoidance::<T>);

        Self {
            agents: world.query(),
            islands: world.query(),
            characters: world.query(),

            world,
            schedule,
            avoidance_schedule,

            marker: std::marker::PhantomData,
        }
    }

    fn tick_state(&mut self) {
        self.agents = self.world.query();
        self.islands = self.world.query();
        self.characters = self.world.query();
    }

    pub fn sample_point(
        &mut self,
        point: Vec3,
        distance: T::SampleDistance,
    ) -> Option<(Vec3, NodeRef)> {
        Archipelago::<T>::sample_point(self.islands(), point, distance)
    }

    pub fn query_sample_point(
        &mut self,
        point: T::Coord,
        point_sample_distance: T::SampleDistance,
    ) -> Result<SampledPoint<T>, SamplePointError> {
        Archipelago::<T>::query_sample_point(self.islands(), point, point_sample_distance)
    }

    pub fn update(&mut self, options: &AgentOptions<T>, delta_time: f32) {
        self.world.insert_resource(*options);
        self.world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(delta_time));

        self.schedule.run(&mut self.world);

        self.tick_state();
    }

    pub fn update_avoidance(&mut self, options: &AgentOptions<T>, delta_time: f32) {
        self.world.insert_resource(*options);
        self.world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(delta_time));

        self.avoidance_schedule.run(&mut self.world);

        self.tick_state();
    }

    pub fn draw_debug(
        &mut self,
        drawer: &mut impl DebugDrawer<T>,
        options: &AgentOptions<T>,
    ) -> Result<(), DebugDrawError> {
        let mut system_state: SystemState<(
            Res<Archipelago<T>>,
            Query<(Entity, &Agent<T>)>,
            Query<(Entity, &Island<T>)>,
        )> = SystemState::new(&mut self.world);

        let (nav, agents, islands) = system_state.get_mut(&mut self.world);
        draw_archipelago_debug::<T>(agents, islands, &nav, drawer, options)
    }
}

impl<T: CoordinateSystem> ArchipelagoWorld<T> {
    pub fn nav(&self) -> &Archipelago<T> {
        self.world.resource()
    }

    pub fn find_path(
        &mut self,
        start_node: NodeRef,
        end_node: NodeRef,
        override_node_type_to_cost: &HashMap<NodeType, f32>,
    ) -> PathResult {
        let mut system_state: SystemState<(ResMut<Archipelago<T>>, Query<(Entity, &Island<T>)>)> =
            SystemState::new(&mut self.world);

        let (mut nav, islands) = system_state.get_mut(&mut self.world);
        nav.find_path(islands, start_node, end_node, override_node_type_to_cost)
    }

    pub fn query_find_path(
        &mut self,
        start: &SampledPoint<T>,
        end: &SampledPoint<T>,
        override_node_type_costs: &HashMap<NodeType, f32>,
    ) -> Result<Vec<T::Coord>, FindPathError> {
        let mut system_state: SystemState<(ResMut<Archipelago<T>>, Query<(Entity, &Island<T>)>)> =
            SystemState::new(&mut self.world);

        let (mut nav, islands) = system_state.get_mut(&mut self.world);
        nav.query_find_path(islands, start, end, override_node_type_costs)
    }

    pub fn node_types(&self) -> impl Iterator<Item = (NodeType, f32)> + '_ {
        self.nav().node_types()
    }

    pub fn add_node_type(&mut self, cost: f32) -> NodeType {
        let mut nav = self.world.resource_mut::<Archipelago<T>>();
        nav.add_node_type(cost).unwrap()
    }

    pub fn set_node_type_cost(&mut self, node_type: NodeType, cost: f32) {
        let mut nav = self.world.resource_mut::<Archipelago<T>>();
        nav.set_node_type_cost(node_type, cost).unwrap();
    }

    pub fn remove_node_type(&mut self, node_type: NodeType) -> bool {
        let mut system_state: SystemState<(ResMut<Archipelago<T>>, Query<(Entity, &Island<T>)>)> =
            SystemState::new(&mut self.world);

        let (mut nav, islands) = system_state.get_mut(&mut self.world);
        nav.remove_node_type(islands, node_type)
    }
}

#[allow(dead_code)]
impl<T: CoordinateSystem> ArchipelagoWorld<T> {
    pub fn add_agent(&mut self, agent: Agent<T>) -> AgentId {
        let id = AgentId(self.world.spawn(agent).id());
        self.tick_state();
        id
    }

    pub fn remove_agent(&mut self, AgentId(entity): AgentId) {
        assert!(self.world.despawn(entity));
    }

    pub fn agent(&self, AgentId(entity): AgentId) -> &Agent<T> {
        self.world.get(entity).unwrap()
    }

    pub fn agent_mut(&mut self, AgentId(entity): AgentId) -> Mut<'_, Agent<T>> {
        self.world.get_mut(entity).unwrap()
    }

    pub fn agents(&mut self) -> Query<'_, '_, (Entity, &'static Agent<T>)> {
        self.agents.query(&self.world)
    }

    pub fn agents_mut(&mut self) -> Query<'_, '_, (Entity, &'static mut Agent<T>)> {
        self.agents.query_mut(&mut self.world)
    }

    pub fn agents_iter(&mut self) -> impl Iterator<Item = (AgentId, &Agent<T>)> {
        self.agents
            .query(&self.world)
            .into_iter()
            .map(|(entity, agent)| (AgentId(entity), agent))
    }

    pub fn agents_iter_mut(&mut self) -> impl Iterator<Item = (AgentId, Mut<'_, Agent<T>>)> {
        self.agents
            .query_mut(&mut self.world)
            .into_iter()
            .map(|(entity, agent)| (AgentId(entity), agent))
    }

    pub fn agent_ids(&mut self) -> Vec<AgentId> {
        self.agents()
            .into_iter()
            .map(|(id, _)| AgentId(id))
            .collect()
    }
}

#[allow(dead_code)]
impl<T: CoordinateSystem> ArchipelagoWorld<T> {
    pub fn add_character(&mut self, character: Character<T>) -> CharacterId {
        let id = CharacterId(self.world.spawn(character).id());
        self.tick_state();
        id
    }

    pub fn remove_character(&mut self, CharacterId(entity): CharacterId) {
        assert!(self.world.despawn(entity));
    }

    pub fn character(&self, CharacterId(entity): CharacterId) -> &Character<T> {
        self.world.get(entity).unwrap()
    }

    pub fn character_mut(&mut self, CharacterId(entity): CharacterId) -> Mut<'_, Character<T>> {
        self.world.get_mut(entity).unwrap()
    }

    pub fn characters(&mut self) -> Query<'_, '_, (Entity, &'static Character<T>)> {
        self.characters.query(&self.world)
    }

    pub fn characters_mut(&mut self) -> Query<'_, '_, (Entity, &'static mut Character<T>)> {
        self.characters.query_mut(&mut self.world)
    }

    pub fn character_ids(&mut self) -> Vec<CharacterId> {
        self.characters()
            .into_iter()
            .map(|(id, _)| CharacterId(id))
            .collect()
    }
}

#[allow(dead_code)]
impl<T: CoordinateSystem> ArchipelagoWorld<T> {
    pub fn add_island(&mut self, island: Island<T>) -> Entity {
        let id = self.world.spawn(island).id();
        self.tick_state();
        id
    }

    pub fn island(&self, entity: IslandId) -> Option<&Island<T>> {
        self.world.get(entity)
    }

    pub fn island_mut(&mut self, entity: IslandId) -> Option<Mut<'_, Island<T>>> {
        self.world.get_mut(entity)
    }

    pub fn remove_island(&mut self, entity: IslandId) -> bool {
        self.world.despawn(entity)
    }

    pub fn islands(&self) -> Query<'_, '_, (Entity, &'static Island<T>)> {
        self.islands.query_manual(&self.world)
    }

    pub fn islands_mut(&mut self) -> Query<'_, '_, (Entity, &'static mut Island<T>)> {
        self.islands.query_mut(&mut self.world)
    }

    pub(crate) fn island_ids(&mut self) -> Vec<IslandId> {
        self.islands().into_iter().map(|(id, _)| id).collect()
    }

    pub(crate) fn islands_len(&mut self) -> usize {
        self.islands().into_iter().len()
    }
}
