use bevy::app::{App, Plugin};
use bevy::asset::{load_internal_asset, Handle};
use bevy::core_pipeline::core_3d::{Transparent3d, CORE_3D_DEPTH_FORMAT};
use bevy::ecs::prelude::*;
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::image::BevyDefault;
use bevy::pbr::{MeshPipeline, MeshPipelineKey, SetMeshViewBindGroup};
use bevy::render::prelude::*;
use bevy::render::render_asset::{
    prepare_assets, PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssetUsages,
    RenderAssets,
};
use bevy::render::render_phase::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::{ExtractedView, RenderLayers, ViewTarget};
use bevy::render::{Extract, Render, RenderApp, RenderSet};
use bytemuck::cast_slice;

const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(7414812681337026784 + 1);

pub struct PainterRenderPlugin;

impl Plugin for PainterRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SHADER_HANDLE, "render.wgsl", Shader::from_wgsl);

        app.init_resource::<PainterHandles>()
            .add_plugins(RenderAssetPlugin::<GpuPainter>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_command::<Transparent3d, PainterRenderCommand>()
            .init_resource::<SpecializedRenderPipelines<PainterPipeline>>()
            .add_systems(
                Render,
                queue
                    .in_set(RenderSet::Queue)
                    .after(prepare_assets::<GpuPainter>),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, extract_painter_data)
            .init_resource::<PainterPipeline>();
    }
}

#[derive(Resource, Default)]
pub struct PainterHandles {
    pub handle: Option<Handle<super::painter::PainterData>>,
}

fn extract_painter_data(mut commands: Commands, handles: Extract<Res<PainterHandles>>) {
    for handle in handles.handle.iter() {
        commands.spawn((*handle).clone_weak());
    }
}

#[derive(Debug, Clone)]
pub struct GpuPainter {
    position_buffer: Buffer,
    index_buffer: Buffer,
    color_buffer: Buffer,
    index_count: u32,
}

impl RenderAsset for GpuPainter {
    type SourceAsset = super::painter::PainterData;
    type Param = SRes<RenderDevice>;

    fn asset_usage(_source_asset: &Self::SourceAsset) -> RenderAssetUsages {
        RenderAssetUsages::all()
    }

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        render_device: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        let position_buffer_data = cast_slice(&source_asset.vertices);
        let position_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: Some("Painter Position Buffer"),
            contents: position_buffer_data,
        });

        let index_buffer_data = cast_slice(&source_asset.indices);
        let index_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::INDEX,
            label: Some("Painter Index Buffer"),
            contents: index_buffer_data,
        });

        let color_buffer_data = cast_slice(&source_asset.colors);
        let color_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: Some("Painter Color Buffer"),
            contents: color_buffer_data,
        });

        Ok(Self {
            index_buffer,
            position_buffer,
            color_buffer,
            index_count: source_asset.indices.len() as u32,
        })
    }
}

struct DrawPainterCommand;

impl<P: PhaseItem> RenderCommand<P> for DrawPainterCommand {
    type ViewQuery = ();
    type ItemQuery = Read<Handle<super::painter::PainterData>>;
    type Param = SRes<RenderAssets<GpuPainter>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        handle: Option<ROQueryItem<'w, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(handle) = handle else {
            return RenderCommandResult::Failure;
        };

        let Some(param) = param.into_inner().get(handle) else {
            return RenderCommandResult::Failure;
        };

        pass.set_index_buffer(param.index_buffer.slice(..), 0, IndexFormat::Uint32);
        pass.set_vertex_buffer(0, param.position_buffer.slice(..));
        pass.set_vertex_buffer(1, param.color_buffer.slice(..));

        pass.draw_indexed(0..param.index_count, 0, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Clone, Resource)]
struct PainterPipeline {
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for PainterPipeline {
    fn from_world(render_world: &mut World) -> Self {
        Self {
            mesh_pipeline: render_world.resource::<MeshPipeline>().clone(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct GizmoPipelineKey {
    view_key: MeshPipelineKey,
}

impl SpecializedRenderPipeline for PainterPipeline {
    type Key = GizmoPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let shader_defs = vec![];

        let format = if key.view_key.contains(MeshPipelineKey::HDR) {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        let view_layout = self
            .mesh_pipeline
            .get_view_layout(key.view_key.into())
            .clone();

        let layout = vec![view_layout];

        RenderPipelineDescriptor {
            label: Some("Painter Pipeline".into()),
            vertex: VertexState {
                shader: SHADER_HANDLE,
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![
                    VertexBufferLayout {
                        array_stride: VertexFormat::Float32x2.size(),
                        step_mode: VertexStepMode::Vertex,
                        attributes: vec![VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    VertexBufferLayout {
                        array_stride: VertexFormat::Float32x4.size(),
                        step_mode: VertexStepMode::Vertex,
                        attributes: vec![VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 1,
                        }],
                    },
                ],
            },
            fragment: Some(FragmentState {
                shader: SHADER_HANDLE,
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout,
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: key.view_key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            push_constant_ranges: vec![],
        }
    }
}

type PainterRenderCommand = (SetItemPipeline, SetMeshViewBindGroup<0>, DrawPainterCommand);

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn queue(
    draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<PainterPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<PainterPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,

    handles: Query<(Entity, &super::painter::PainterHandle)>,
    assets: Res<RenderAssets<GpuPainter>>,

    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    mut views: Query<(Entity, &ExtractedView, Option<&RenderLayers>)>,
) {
    let draw_function = draw_functions
        .read()
        .get_id::<PainterRenderCommand>()
        .unwrap();

    for (view_entity, view, _render_layers) in &mut views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_hdr(view.hdr);

        // let render_layers = render_layers.unwrap_or_default();
        for (entity, handle) in &handles {
            // if !config.render_layers.intersects(render_layers) {
            //     continue;
            // }

            let Some(_) = assets.get(handle.id()) else {
                continue;
            };

            let key = GizmoPipelineKey { view_key };

            transparent_phase.add(Transparent3d {
                entity,
                draw_function,
                pipeline: pipelines.specialize(&pipeline_cache, &pipeline, key),
                distance: 0.,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::NONE,
            });
        }
    }
}
