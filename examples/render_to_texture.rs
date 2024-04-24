//! Shows how to render to a texture. Useful for mirrors, UI, or exporting images.

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_mod_picking::prelude::*;
use bevy_render::camera::NormalizedRenderTarget;
use picking_core::pointer::{InputMove, Location};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, sys)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    let image_handle = {
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        image.resize(size);
        images.add(image)
    };

    // render to texture
    {
        let render_layer = RenderLayers::layer(1);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(4.0, 4.0, 4.0)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.8, 0.7, 0.6),
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            },
            render_layer,
            PickableBundle::default(),
            On::<Pointer<Move>>::run(|| {
                println!("move");
            }),
        ));

        commands.spawn((
            PointLightBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
                ..default()
            },
            render_layer,
        ));

        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    order: -1,
                    target: image_handle.clone().into(),
                    clear_color: Color::WHITE.into(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(5.0, 5.0, 15.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            render_layer,
        ));
    }

    // render quad with texture
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Rectangle::from_size(Vec2::splat(8.0))),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(image_handle.clone()),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 1.5),
            ..default()
        },
        Pickable::default(),
        focus::PickingInteraction::default(),
        TargetTextureHandle(image_handle.clone()),
        On::<Pointer<Move>>::run(
            |event: Listener<Pointer<Move>>,
             handle: Query<&TargetTextureHandle>,
             mut pointer_move: EventWriter<InputMove>| {
                let target_texture_handle = handle.get(event.target).unwrap().0.clone();
                pointer_move.send(InputMove {
                    pointer_id: PointerId::Mouse,
                    location: Location {
                        target: NormalizedRenderTarget::Image(target_texture_handle),
                        position: Vec2::ZERO,
                    },
                    delta: Vec2::ZERO,
                });
            },
        ),
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct TargetTextureHandle(Handle<Image>);

fn sys(mut listener: EventReader<InputMove>) {
    for input_move in listener.read() {
        println!("{:?}", input_move.location.target);
    }
}
