use bevy::prelude::{MessageWriter, Res, ResMut, Resource};
use bevy::render::MainWorld;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache};
use bevy::window::RequestRedraw;

#[derive(Resource, Default, Debug)]
pub(crate) struct PipelinesModCount(usize);

/// Pipeline checking system to increase the mod count to trigger refresh.
pub(crate) fn check_pipelines_ready_system(
    mut main_world: ResMut<MainWorld>,
    cache: Res<PipelineCache>,
) {
    if let Some(mut mod_count) = main_world.get_resource_mut::<PipelinesModCount>()
        && cache
            .pipelines()
            .any(|p| !matches!(p.state, CachedPipelineState::Ok(_)))
    {
        mod_count.0 = mod_count.0.wrapping_add(1);
    }
}

/// Refresh system listening to the change in pipeline mod count.
pub(crate) fn pipeline_refresh_system(mut redraw_request_writer: MessageWriter<RequestRedraw>) {
    redraw_request_writer.write(RequestRedraw);
}
