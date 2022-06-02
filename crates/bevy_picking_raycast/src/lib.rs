use bevy::prelude::*;
use bevy_picking_core::{simple_criteria, PickingSettings, PickingSystem};

#[derive(Debug, Clone, Hash, PartialEq, Eq, SystemLabel)]
pub enum RaycastSystem {
    UpdatePickSourcePositions,
    BuildRays,
    UpdateRaycast,
    UpdateIntersections,
}

/// A type alias for the concrete [RayCastMesh](bevy_mod_raycast::RayCastMesh) type used for Picking.
pub type RaycastTarget = bevy_mod_raycast::RayCastMesh<PickingRaycastSet>;
/// A type alias for the concrete [RayCastSource](bevy_mod_raycast::RayCastSource) type used for Picking.
pub type RaycastSource = bevy_mod_raycast::RayCastSource<PickingRaycastSet>;

/// This unit struct is used to tag the generic ray casting types `RayCastMesh` and
/// `RayCastSource`. This means that all Picking ray casts are of the same type. Consequently, any
/// meshes or ray sources that are being used by the picking plugin can be used by other ray
/// casting systems because they will have distinct types, e.g.: `RayCastMesh<PickingRaycastSet>`
/// vs. `RayCastMesh<MySuperCoolRaycastingType>`, and as such wil not result in collisions.
pub struct PickingRaycastSet;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .with_run_criteria(|state: Res<PickingSettings>| {
                    simple_criteria(state.enable_picking)
                })
                .with_system(
                    bevy_mod_raycast::build_rays::<PickingRaycastSet>
                        .label(RaycastSystem::BuildRays)
                        .after(RaycastSystem::UpdatePickSourcePositions)
                        .before(RaycastSystem::UpdateRaycast),
                )
                .with_system(
                    bevy_mod_raycast::update_raycast::<PickingRaycastSet>
                        .label(RaycastSystem::UpdateRaycast)
                        .before(RaycastSystem::UpdateIntersections),
                )
                .with_system(
                    bevy_mod_raycast::update_intersections::<PickingRaycastSet>
                        .label(RaycastSystem::UpdateIntersections)
                        .before(PickingSystem::PauseForBlockers)
                        .before(PickingSystem::InitialHighlights),
                ),
        );
    }
}

/// Update Screenspace ray cast sources with the current mouse position
pub fn update_pick_source_positions(
    touches_input: Res<Touches>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut cursor: EventReader<CursorMoved>,
    mut pick_source_query: Query<(&mut RaycastSource, Option<&Camera>)>,
) {
    for (mut pick_source, option_update_picks, option_camera) in &mut pick_source_query.iter_mut() {
        let (mut update_picks, cursor_latest) = match get_inputs(
            &windows,
            &images,
            option_camera,
            option_update_picks,
            &mut cursor,
            &touches_input,
        ) {
            Some(value) => value,
            None => return,
        };
        match *update_picks {
            UpdatePicks::EveryFrame(cached_cursor_pos) => {
                match cursor_latest {
                    Some(cursor_moved) => {
                        pick_source.cast_method = RayCastMethod::Screenspace(cursor_moved);
                        *update_picks = UpdatePicks::EveryFrame(cursor_moved);
                    }
                    None => pick_source.cast_method = RayCastMethod::Screenspace(cached_cursor_pos),
                };
            }
            UpdatePicks::OnMouseEvent => match cursor_latest {
                Some(cursor_moved) => {
                    pick_source.cast_method = RayCastMethod::Screenspace(cursor_moved)
                }
                None => continue,
            },
        };
    }
}

fn get_inputs<'a>(
    windows: &Res<Windows>,
    images: &Res<Assets<Image>>,
    option_camera: Option<&Camera>,
    option_update_picks: Option<Mut<'a, UpdatePicks>>,
    cursor: &mut EventReader<CursorMoved>,
    touches_input: &Res<Touches>,
) -> Option<(Mut<'a, UpdatePicks>, Option<Vec2>)> {
    let camera = option_camera?;
    let update_picks = option_update_picks?;
    let height = camera.target.get_logical_size(windows, images)?.y;
    let cursor_latest = match cursor.iter().last() {
        Some(cursor_moved) => {
            if let RenderTarget::Window(window) = camera.target {
                if cursor_moved.id == window {
                    Some(cursor_moved.position)
                } else {
                    None
                }
            } else {
                None
            }
        }
        None => touches_input.iter().last().map(|touch| {
            Vec2::new(
                touch.position().x as f32,
                height - touch.position().y as f32,
            )
        }),
    };
    Some((update_picks, cursor_latest))
}
