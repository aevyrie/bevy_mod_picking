use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use std::f32::consts::FRAC_PI_2;

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

/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1200.0, -400.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // Spawn cubes
    for x in -2..=2 {
        let z = 0.5 + x as f32 * 0.1;
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::default()).into(),
                transform: Transform::from_xyz(x as f32 * 200.0, 0.0, z)
                    .with_scale(Vec3::splat(100.)),
                material: materials.add(Color::hsl(0.0, 1.0, z)),
                ..default()
            },
            PickableBundle::default(), // <- Makes the mesh pickable.
            On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
            On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
            On::<Pointer<Drag>>::run(on_drag),
            On::<Pointer<Drop>>::commands_mut(|event, commands| {
                commands.entity(event.dropped).insert(Spin(FRAC_PI_2)); // Spin dropped entity
                commands.entity(event.target).insert(Spin(-FRAC_PI_2)); // Spin dropped-on entity
            }),
        ));
    }
}

fn on_drag(
    listener: Listener<Pointer<Drag>>,
    mut transforms: Query<(&mut Transform, &GlobalTransform)>,
    windows: Query<Entity, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(primary_window) = windows.get_single() else {
        return;
    };
    let target = &listener.pointer_location.target;
    let Some((cam, cam_trans)) = cameras.iter().find(|(c, _)| {
        c.is_active && c.target.normalize(Some(primary_window)).as_ref() == Some(target)
    }) else {
        return;
    };

    let Ok((mut t, gt)) = transforms.get_mut(listener.target) else {
        return;
    };
    let Some(prev) = cam.world_to_viewport(cam_trans, gt.translation()) else {
        return;
    };

    let new = prev + listener.event.delta;
    let Some(ray) = cam.viewport_to_world(cam_trans, new) else {
        return;
    };
    if let Some(dist) = ray.intersect_plane(Vec3::Y, InfinitePlane3d { normal: Dir3::Y }) {
        t.translation += ray.get_point(dist) - gt.translation();
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
