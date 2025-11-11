use crate::{
    camera::camera_ext, camera::main_camera::MainCamera, rendering::tile::TileModState,
    rendering::tiled_image::TiledImage,
};
use bevy::{prelude::*, ui::RelativeCursorPosition};

#[derive(Component)]
pub(crate) struct MinimapViewRect;

#[derive(Component)]
pub(crate) struct MinimapImage;

const BORDER_SIZE: f32 = 2.0;
const MINIMAP_SIZE: f32 = 200.0;
const THUMBNAIL_SIZE: f32 = MINIMAP_SIZE - 2.0 * BORDER_SIZE;

pub(crate) fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiled_image: Single<&TiledImage>,
) {
    let container = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::End,
        ..default()
    };

    let thumbnail_container = (
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
        BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
        Node {
            width: Val::Px(MINIMAP_SIZE),
            height: Val::Px(MINIMAP_SIZE),
            align_self: AlignSelf::End,
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        },
    );

    let (thumbnail_url, thumbnail_size) = tiled_image.get_image_thumbnail(256);
    let (thumbnail_scale, offset) =
        get_thumbnail_scale_and_offset(Rect::from_corners(Vec2::ZERO, thumbnail_size));
    let thumbnail_rect = Rect::from_corners(
        Vec2::ZERO + offset,
        thumbnail_size * thumbnail_scale + offset,
    );

    let thumbnail_image = (
        MinimapImage,
        Button,
        RelativeCursorPosition::default(),
        Node {
            left: Val::Px(thumbnail_rect.min.x),
            top: Val::Px(thumbnail_rect.min.y),
            width: Val::Px(thumbnail_rect.width()),
            height: Val::Px(thumbnail_rect.height()),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode {
            image: asset_server.load(thumbnail_url),
            ..default()
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
        container,
        children![(thumbnail_container, children![thumbnail_image, view_rect])],
    ));
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

pub(crate) fn update_view_rect_system(
    mut view_rect: Single<&mut Node, With<MinimapViewRect>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
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

pub(crate) fn mouse_input_system(
    interaction: Single<&Interaction, (Changed<Interaction>, With<MinimapImage>)>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    cursor_query: Query<&RelativeCursorPosition>,
    camera_query: Single<&mut Transform, With<MainCamera>>,
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

    mouse.clear();
}
