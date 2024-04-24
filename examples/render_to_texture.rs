//! Renders the 3d scene to a texture, displays it in egui as a viewport, and adds picking support.

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    transform,
};
use bevy_egui::*;
use bevy_mod_picking::prelude::*;
use picking_core::pointer::{InputMove, InputPress, Location};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultPickingPlugins.build().disable::<InputPlugin>(),
            ViewportInputPlugin,
            EguiPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (ui, spin_cube))
        .run();
}

/// Send this event to spawn a new viewport.
#[derive(Event, Default)]
pub struct SpawnViewport;

/// Replaces bevy_mod_picking's default `InputPlugin`, and replaces it with inputs driven by egui,
/// sending picking inputs when a pointer is over a viewport that has been rendered to a texture and
/// laid out inside the ui.
struct ViewportInputPlugin;

impl Plugin for ViewportInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnViewport>()
            .add_systems(First, Self::send_mouse_clicks)
            // Default input plugin is disabled, we need to spawn a mouse pointer.
            .add_systems(Startup, input::mouse::spawn_mouse_pointer)
            .add_systems(Update, spawn_viewport.run_if(on_event::<SpawnViewport>()));
    }
}

impl ViewportInputPlugin {
    fn send_mouse_clicks(
        mut mouse_inputs: Res<ButtonInput<MouseButton>>,
        mut pointer_press: EventWriter<InputPress>,
    ) {
        if mouse_inputs.just_pressed(MouseButton::Left) {
            pointer_press.send(InputPress {
                pointer_id: PointerId::Mouse,
                direction: pointer::PressDirection::Down,
                button: PointerButton::Primary,
            });
        } else if mouse_inputs.just_released(MouseButton::Left) {
            pointer_press.send(InputPress {
                pointer_id: PointerId::Mouse,
                direction: pointer::PressDirection::Up,
                button: PointerButton::Primary,
            });
        }
    }
}

#[derive(Component)]
struct EguiViewport(Handle<Image>);

fn ui(
    mut egui_contexts: EguiContexts,
    egui_viewports: Query<(Entity, &EguiViewport)>,
    mut pointer_move: EventWriter<InputMove>,
    mut spawn_viewport: EventWriter<SpawnViewport>,
) {
    egui::TopBottomPanel::top("menu_panel").show(egui_contexts.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            if ui.button("New Viewport").clicked() {
                spawn_viewport.send_default();
            }
        });
    });

    // Draw every viewport in a window. This isn't as robust as it could be for the sake of
    // demonstration. This only works if the render target and egui texture are rendered at the same
    // resolution, and this completely ignores touch inputs and treats everything as a mouse input.
    for (viewport_entity, egui_viewport) in &egui_viewports {
        let viewport_texture = &egui_viewport.0;
        let viewport_texture_id = egui_contexts.add_image(viewport_texture.clone_weak());

        egui::Window::new(format!("Viewport {:?}", viewport_entity))
            .id(egui::Id::new(viewport_entity))
            .show(egui_contexts.ctx_mut(), |ui| {
                // Draw the texture and get a response to check if a pointer is interacting
                let viewport_response = ui.add(egui::widgets::Image::new(
                    egui::load::SizedTexture::new(viewport_texture_id, [256.0, 256.0]),
                ));

                if let Some(pointer_pos_window) = viewport_response.hover_pos() {
                    // Compute the position of the pointer relative to the texture.
                    let pos = pointer_pos_window - viewport_response.rect.min;
                    pointer_move.send(InputMove {
                        pointer_id: PointerId::Mouse,
                        location: Location {
                            target: bevy_render::camera::NormalizedRenderTarget::Image(
                                viewport_texture.clone_weak(),
                            ),
                            position: Vec2::new(pos.x, pos.y),
                        },
                        delta: Vec2::ZERO,
                    });
                }
            });
    }
}

/// Spawn the light and cube mesh.
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    },));

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
        PickableBundle::default(),
    ));
}

/// Spawn a new camera to use as a viewport in egui on demand.
fn spawn_viewport(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {
    let size = Extent3d {
        width: 256,
        height: 256,
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

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 1_000_001,
                target: image_handle.clone().into(),
                clear_color: Color::WHITE.into(),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                time.elapsed_seconds().cos() * 10.0,
                0.0,
                time.elapsed_seconds().sin() * 10.0,
            ))
            .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        EguiViewport(image_handle.clone()),
    ));
}

fn spin_cube(mut cube: Query<&mut Transform, With<Handle<Mesh>>>, time: Res<Time>) {
    cube.single_mut().rotate_x(time.delta_seconds() * 0.1)
}
