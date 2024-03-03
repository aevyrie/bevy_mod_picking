use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(low_latency_window_plugin()),
        DefaultPickingPlugins,
        SpinPlugin,
    ))
    .insert_resource(DebugPickingMode::Normal)
    .add_systems(Startup, setup);
    #[cfg(feature = "backend_egui")]
    app.add_plugins(bevy_egui::EguiPlugin);
    app.run();
}

/// Set up a simple 2D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
    // Spawn squares
    for x in -2..=2 {
        let z = 0.5 + x as f32 * 0.1;
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::default()).into(),
                transform: Transform::from_xyz(x as f32 * 200.0, 0.0, z)
                    .with_scale(Vec3::splat(100.)),
                material: materials.add(ColorMaterial::from(Color::hsl(0.0, 1.0, z))),
                ..default()
            },
            PickableBundle::default(), // <- Makes the mesh pickable.
            On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
            On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
            On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                transform.translation.x += drag.delta.x; // Make the square follow the mouse
                transform.translation.y -= drag.delta.y;
            }),
            On::<Pointer<Drop>>::commands_mut(|event, commands| {
                commands.entity(event.dropped).insert(Spin(FRAC_PI_2)); // Spin dropped entity
                commands.entity(event.target).insert(Spin(-FRAC_PI_2)); // Spin dropped-on entity
            }),
        ));
    }
}

pub struct SpinPlugin;

impl Plugin for SpinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spin)
            .add_systems(Update, spin_cleanup);
    }
}

#[derive(Component)]
struct Spin(f32);

fn spin(mut square: Query<(&mut Spin, &mut Transform)>) {
    for (mut spin, mut transform) in square.iter_mut() {
        transform.rotation = Quat::from_rotation_z(spin.0);
        let delta = -spin.0.clamp(-1.0, 1.0) * 0.05;
        spin.0 += delta;
    }
}

fn spin_cleanup(mut square: Query<(Entity, &Spin, &mut Transform)>, mut commands: Commands) {
    for (entity, spin, mut transform) in square.iter_mut() {
        if spin.0.abs().le(&0.001) {
            transform.rotation = Quat::default(); // <- reset the rotation to zero when it's visually neglible
            commands.entity(entity).remove::<Spin>(); // <- remove the component so it's stopped updating
        }
    }
}
