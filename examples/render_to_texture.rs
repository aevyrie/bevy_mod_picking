//! Renders the 3d scene to a texture, displays it in egui as a viewport, and adds picking support.

use bevy::{prelude::*, render::render_resource::*};
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
        .add_systems(Update, (ui, animate_scene))
        .run();
}

const VIEWPORT_SIZE: u32 = 256;

/// Send this event to spawn a new viewport.
#[derive(Event, Default)]
pub struct SpawnViewport;

/// Replaces bevy_mod_picking's default `InputPlugin` with inputs driven by egui, sending picking
/// inputs when a pointer is over a viewport that has been rendered to a texture in the ui.
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
        mouse_inputs: Res<ButtonInput<MouseButton>>,
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
struct EguiViewport {
    bevy: Handle<Image>,
    egui: egui::TextureId,
}

fn ui(
    mut commands: Commands,
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

    // Draw each viewport in a window. This isn't as robust as it could be for the sake of
    // demonstration. This only works if the render target and egui texture are rendered at the same
    // resolution, and this completely ignores touch inputs and treats everything as a mouse input.
    for (viewport_entity, egui_viewport) in &egui_viewports {
        let mut is_open = true;
        egui::Window::new(format!("{:?}", viewport_entity))
            .id(egui::Id::new(viewport_entity))
            .open(&mut is_open)
            .show(egui_contexts.ctx_mut(), |ui| {
                // Draw the texture and get a response to check if a pointer is interacting
                let viewport_response =
                    ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
                        egui_viewport.egui,
                        [VIEWPORT_SIZE as f32, VIEWPORT_SIZE as f32],
                    )));

                if let Some(pointer_pos_window) = viewport_response.hover_pos() {
                    // Compute the position of the pointer relative to the texture.
                    let pos = pointer_pos_window - viewport_response.rect.min;
                    pointer_move.send(InputMove {
                        pointer_id: PointerId::Mouse,
                        location: Location {
                            target: bevy_render::camera::NormalizedRenderTarget::Image(
                                egui_viewport.bevy.clone_weak(),
                            ),
                            position: Vec2::new(pos.x, pos.y),
                        },
                        delta: Vec2::ZERO,
                    });
                }
            });
        if !is_open {
            commands.entity(viewport_entity).despawn_recursive();
        }
    }
}

/// Spawn a new camera to use as a viewport in egui on demand.
fn spawn_viewport(
    mut egui_contexts: EguiContexts,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {
    let size = Extent3d {
        width: VIEWPORT_SIZE,
        height: VIEWPORT_SIZE,
        ..default()
    };

    let viewport_handle = {
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
    let viewport_texture_id = egui_contexts.add_image(viewport_handle.clone_weak());

    let elapsed = time.elapsed_seconds();
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                target: viewport_handle.clone().into(),
                ..default()
            },
            transform: Transform::from_translation(
                Vec3::new(elapsed.cos(), 0.0, elapsed.sin()) * 5.0,
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        EguiViewport {
            bevy: viewport_handle.clone_weak(),
            egui: viewport_texture_id,
        },
    ));
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        brightness: 500.0,
        ..default()
    });
    commands.spawn((PointLightBundle {
        transform: Transform::from_translation(Vec3::new(2.0, 1.0, 5.0)),
        ..default()
    },));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(StandardMaterial::default()),
            ..default()
        },
        PickableBundle::default(),
    ));
}

fn animate_scene(
    mut cube: Query<&mut Transform, (With<Handle<Mesh>>, Without<PointLight>)>,
    mut light: Query<&mut Transform, With<PointLight>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    cube.single_mut().rotate_x(dt * 0.2);
    light
        .single_mut()
        .rotate_around(Vec3::ZERO, Quat::from_rotation_y(dt * 0.8));
}
