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
    if cursor_events.cursor_event_reader.latest(&cursor).is_some() {
        println!("Top entities:\n{:#?}", pick_state.top_all())
    }
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
        let transform_new = top_pick.intersection.to_transform();
        for mut transform in &mut query.iter_mut() {
            *transform = Transform::from_matrix(transform_new);
        }
        for mut draw in &mut visibility_query.iter_mut() {
            draw.is_visible = true;
        }
    }
    if pick_state.top_all().is_empty() {
        for mut draw in &mut visibility_query.iter_mut() {
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
            material: debug_matl.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            let mut transform =
                Transform::from_translation(Vec3::new(0.0, cube_size * cube_tail_scale, 0.0));
            transform.apply_non_uniform_scale(Vec3::from([1.0, cube_tail_scale, 1.0]));

            // child cube
            parent
                .spawn(PbrComponents {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
                    material: debug_matl,
                    transform,
                    ..Default::default()
                })
                .with(DebugCursorMesh);
        })
        .with(DebugCursor)
        .with(DebugCursorMesh);
}
