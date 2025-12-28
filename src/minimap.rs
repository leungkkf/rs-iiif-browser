use crate::{
    camera::camera_ext, camera::main_camera::MainCamera2d, rendering::tile::TileModState,
    rendering::tiled_image::TiledImage,
};
use bevy::{
    camera::visibility::Visibility,
    color::Srgba,
    image::TRANSPARENT_IMAGE_HANDLE,
    prelude::{
        Add, AlignSelf, AssetServer, BackgroundColor, BorderColor, Button, Camera, Changed, Color,
        Commands, Component, Display, Entity, GlobalTransform, ImageNode, Interaction,
        JustifyContent, MessageWriter, Node, On, PositionType, Query, Rect, Remove, Res, ResMut,
        Result, Single, SpawnRelated, Transform, UiRect, Val, Vec2, With, children, default, info,
    },
    ui::RelativeCursorPosition,
    window::RequestRedraw,
};

#[derive(Component)]
pub(crate) struct MinimapContainer;

#[derive(Component)]
pub(crate) struct MinimapViewRect;

#[derive(Component)]
pub(crate) struct MinimapImage;

const BORDER_SIZE: f32 = 2.0;
const MINIMAP_SIZE: f32 = 200.0;
const THUMBNAIL_SIZE: f32 = MINIMAP_SIZE - 2.0 * BORDER_SIZE;
const MINIMAP_ALPHA: f32 = 0.75;

/// Set up the minimap using Bevy UI.
pub(crate) fn setup(mut commands: Commands) {
    let container = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::End,
        ..default()
    };

    let thumbnail_container = (
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, MINIMAP_ALPHA)),
        BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
        Node {
            width: Val::Px(MINIMAP_SIZE),
            height: Val::Px(MINIMAP_SIZE),
            align_self: AlignSelf::End,
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        },
    );

    let thumbnail_image = (
        MinimapImage,
        Button,
        RelativeCursorPosition::default(),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode {
            color: Color::Srgba(Srgba::new(1.0, 1.0, 1.0, MINIMAP_ALPHA)),
            ..ImageNode::default()
        },
    );

    let view_rect = (
        MinimapViewRect,
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        BorderColor::all(Color::srgba(0.0, 0.5, 0.5, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            border: UiRect::all(Val::Px(BORDER_SIZE)),
            display: Display::None,
            ..Default::default()
        },
    );

    commands.spawn((
        MinimapContainer,
        container,
        children![(thumbnail_container, children![thumbnail_image, view_rect])],
        Visibility::Hidden,
    ));
}

/// Triggered when the tiled image is removed to clean up.
pub(crate) fn on_remove_tiled_image(
    remove: On<Remove, TiledImage>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
    mut minimap_image: Single<&mut ImageNode, With<MinimapImage>>,
    mut commands: Commands,
    minimap_container_query: Single<Entity, With<MinimapContainer>>,
) -> Result {
    info!("Tiled image removed (minimap). {:?}", remove.entity);

    // Clean up the minimap.
    minimap_image.image = TRANSPARENT_IMAGE_HANDLE;

    // Trigger an update.
    redraw_request_writer.write(RequestRedraw);

    let minimap_container_entity = minimap_container_query.into_inner();

    commands
        .entity(minimap_container_entity)
        .insert((Visibility::Hidden,));

    Ok(())
}

/// Triggered when tiled image is added to update the minimap.
pub(crate) fn on_add_tiled_image(
    add: On<Add, TiledImage>,
    minimap_image_query: Single<(&mut ImageNode, &mut Node), With<MinimapImage>>,
    tiled_image: Single<&TiledImage>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    minimap_container_query: Single<Entity, With<MinimapContainer>>,
) {
    info!("Tiled image added (minimap). {:?}", add.entity);

    let (thumbnail_url, thumbnail_size) = tiled_image.get_image_thumbnail(256);
    let (thumbnail_scale, offset) =
        get_thumbnail_scale_and_offset(Rect::from_corners(Vec2::ZERO, thumbnail_size));
    let thumbnail_rect = Rect::from_corners(
        Vec2::ZERO + offset,
        thumbnail_size * thumbnail_scale + offset,
    );

    let minimap_container_entity = minimap_container_query.into_inner();

    commands
        .entity(minimap_container_entity)
        .insert((Visibility::Visible,));

    let (mut minimap_image, mut minimap_node) = minimap_image_query.into_inner();

    minimap_image.image = asset_server.load(thumbnail_url);
    minimap_node.left = Val::Px(thumbnail_rect.min.x);
    minimap_node.top = Val::Px(thumbnail_rect.min.y);
    minimap_node.width = Val::Px(thumbnail_rect.width());
    minimap_node.height = Val::Px(thumbnail_rect.height());
}

fn get_thumbnail_scale_and_offset(image_size: Rect) -> (f32, Vec2) {
    let scale = THUMBNAIL_SIZE / image_size.width().max(image_size.height());

    (
        scale,
        Vec2::new(
            (THUMBNAIL_SIZE - scale * image_size.width()) / 2.0,
            (THUMBNAIL_SIZE - scale * image_size.height()) / 2.0,
        ),
    )
}

/// Update the main camera rect in the minimap.
pub(crate) fn update_view_rect_system(
    mut view_rect: Single<&mut Node, With<MinimapViewRect>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera2d>>,
    tiled_image: Single<&TiledImage>,
) {
    let (camera, global_transform) = camera_query.into_inner();

    let Some((world_pos_min, world_pos_max)) =
        camera_ext::get_world_viewport_rect(camera, global_transform)
    else {
        return;
    };

    // Convert the viewport in world space to image viewport in image space.
    let image_max_size = tiled_image.get_image_max_size_rect();
    let image_pos_min = tiled_image.world_to_image(world_pos_min);
    let image_pos_max = tiled_image.world_to_image(world_pos_max);

    // Scale to the thumbnail size and add into the offset.
    let (scale, offset) = get_thumbnail_scale_and_offset(image_max_size);

    // Bound by the thumbnail node.
    let bounded_view_rect = Rect::from_corners(
        image_pos_min * scale + offset,
        image_pos_max * scale + offset,
    )
    .intersect(Rect::from_corners(
        Vec2::new(0.0, 0.0),
        Vec2::new(THUMBNAIL_SIZE, THUMBNAIL_SIZE),
    ));

    view_rect.left = Val::Px(bounded_view_rect.min.x);
    view_rect.top = Val::Px(bounded_view_rect.min.y);
    view_rect.width = Val::Px(bounded_view_rect.width());
    view_rect.height = Val::Px(bounded_view_rect.height());
    view_rect.display = Display::Block;
}

/// Check if the bevy UI has mouse input.
pub(crate) fn ui_has_mouse_input(cursor_query: Query<&RelativeCursorPosition>) -> bool {
    if let Ok(cursor) = cursor_query.single()
        && cursor.cursor_over
    {
        true
    } else {
        false
    }
}

/// Handle the mouse events of the minimap.
pub(crate) fn mouse_input_system(
    interaction: Single<&Interaction, (Changed<Interaction>, With<MinimapImage>)>,
    cursor_query: Query<&RelativeCursorPosition>,
    camera_query: Single<&mut Transform, With<MainCamera2d>>,
    tiled_image: Single<&TiledImage>,
    mut tile_mod_state: ResMut<TileModState>,
) {
    if let Ok(cursor) = cursor_query.single() {
        if !cursor.cursor_over || **interaction != Interaction::Pressed {
            return;
        }
        let Some(cursor) = cursor.normalized else {
            return;
        };
        let image_pos =
            tiled_image.get_image_max_size_rect().max * Vec2::new(cursor.x + 0.5, cursor.y + 0.5);

        let world_pos = tiled_image.image_to_world(image_pos);

        let mut transform = camera_query.into_inner();

        transform.translation = world_pos;
        tile_mod_state.invalidate();
    }
}
