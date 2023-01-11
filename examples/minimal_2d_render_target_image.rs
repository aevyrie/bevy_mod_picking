use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::MaterialMesh2dBundle,
};
use bevy_mod_picking::{
    DebugEventsPickingPlugin, DefaultPickingPlugins, PickableBundle, PickingCameraBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds picking, interaction, and highlighting
        .add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_startup_system(setup)
        .run();
}

/// set up a simple 2D scene for a camera with RenderTarget::Image
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = windows.primary();
    let scale_factor = window.scale_factor() as f32;
    let window_width = window.width() as f32;
    let window_height = window.height() as f32;
    let size = Extent3d {
        width: (window_width * scale_factor) as u32,
        height: (window_height * scale_factor) as u32,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);

    // camera to render image
    commands.spawn(
        (
            Camera2dBundle {
                camera: Camera {
                    target: RenderTarget::Image(image_handle.clone()),
                    ..Default::default()
                },
                ..Default::default()
            },
            PickingCameraBundle::default(),
        ), // <- Sets the camera to use for picking.
    );

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_scale(Vec3::splat(128.)),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            ..default()
        },
        PickableBundle::default(), // <- Makes the mesh pickable.
    ));

    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(
                    window_width,
                    window_height,
                ))))
                .into(),
            material: materials.add(ColorMaterial::from(image_handle.clone())),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        post_processing_pass_layer,
    ));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // renders after the first main camera which has default value: 0.
                priority: 1,
                ..default()
            },
            ..Camera2dBundle::default()
        },
        post_processing_pass_layer,
    ));
}
