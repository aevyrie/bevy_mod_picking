use super::*;
use bevy::prelude::*;

pub struct DebugPickingPlugin;
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<CursorEvents>()
            .add_startup_system(setup_debug_cursor.system())
            .add_system(update_debug_cursor_position.system())
            .add_system(get_picks.system());
    }
}

pub struct CursorEvents {
    cursor_event_reader: EventReader<CursorMoved>,
}

impl Default for CursorEvents {
    fn default() -> Self {
        CursorEvents {
            cursor_event_reader: EventReader::default(),
        }
    }
}

fn get_picks(
    pick_state: Res<PickState>,
    mut cursor_events: ResMut<CursorEvents>,
    cursor: Res<Events<CursorMoved>>,
) {
    match cursor_events.cursor_event_reader.latest(&cursor) {
        Some(_) => println!("Top entities:\n{:#?}", pick_state.top_all()),
        None => {},
    };
}

struct DebugCursor;

struct DebugCursorMesh;

/// Updates the 3d cursor to be in the pointed world coordinates
fn update_debug_cursor_position(
    pick_state: Res<PickState>,
    mut query: Query<With<DebugCursor, &mut Transform>>,
    mut visibility_query: Query<With<DebugCursorMesh, &mut Draw>>,
) {
    // Set the cursor translation to the top pick's world coordinates
    for (_group, top_pick) in pick_state.top_all() {
        let position = top_pick.position();
        let normal = top_pick.normal();
        let up = Vec3::from([0.0, 1.0, 0.0]);
        let axis = up.cross(*normal).normalize();
        let angle = up.dot(*normal).acos();
        let epsilon = 0.0001;
        let new_rotation = if angle.abs() > epsilon {
            Quat::from_axis_angle(axis, angle)
        } else {
            Quat::default()
        };
        let transform_new = Mat4::from_rotation_translation(new_rotation, *position);
        for mut transform in &mut query.iter() {
            *transform.value_mut() = transform_new;
        }
        for mut draw in &mut visibility_query.iter() {
            draw.is_visible = true;
        }
    }
    if pick_state.top_all().is_empty() {
        for mut draw in &mut visibility_query.iter() {
            draw.is_visible = false;
        }
    }
}

/// Start up system to create 3d Debug cursor
fn setup_debug_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_matl = materials.add(StandardMaterial {
        albedo: Color::rgb(0.0, 1.0, 0.0),
        shaded: false,
        ..Default::default()
    });
    let cube_size = 0.02;
    let cube_tail_scale = 20.0;
    let ball_size = 0.08;
    commands
        // cursor
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 4,
                radius: ball_size,
            })),
            material: debug_matl,
            ..Default::default()
        })
        .with_children(|parent| {
            // child cube
            parent
                .spawn(PbrComponents {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
                    material: debug_matl,
                    transform: Transform::from_non_uniform_scale(Vec3::from([
                        1.0,
                        cube_tail_scale,
                        1.0,
                    ]))
                    .with_translation(Vec3::new(
                        0.0,
                        cube_size * cube_tail_scale,
                        0.0,
                    )),
                    ..Default::default()
                })
                .with(DebugCursorMesh);
        })
        .with(DebugCursor)
        .with(DebugCursorMesh);
}
