//! Text and on-screen debugging tools

use bevy_picking_core::{debug, focus::HoverMap};
use picking_core::{events::DragMap, pointer::Location};

use crate::*;
use bevy::{asset::load_internal_binary_asset, prelude::*, utils::Uuid};

const DEBUG_FONT_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(
    Uuid::from_u128(200742528088501825055247279035227365784),
    436509473926038,
);

fn font_loader(bytes: &[u8]) -> Font {
    Font::try_from_bytes(bytes.to_vec()).unwrap()
}

/// Logs events for debugging
#[derive(Debug, Default, Clone)]
pub struct DebugPickingPlugin {
    /// Suppresses noisy events like `Move` and `Drag` when set to `false`
    pub noisy: bool,
}
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut App) {
        let noisy_debug = self.noisy;

        load_internal_binary_asset!(app, DEBUG_FONT_HANDLE, "FiraMono-Medium.ttf", font_loader);

        app.init_resource::<debug::Frame>()
            .add_systems(First, debug::increment_frame)
            .add_systems(
                PreUpdate,
                input::debug::print
                    .before(picking_core::PickSet::Backend)
                    .run_if(move || noisy_debug),
            )
            .add_systems(
                PreUpdate,
                (
                    debug::print::<events::Over>,
                    debug::print::<events::Out>,
                    debug::print::<events::Down>,
                    debug::print::<events::Up>,
                    debug::print::<events::Click>,
                    debug::print::<events::Move>.run_if(move || noisy_debug),
                    debug::print::<events::DragStart>,
                    debug::print::<events::Drag>.run_if(move || noisy_debug),
                    debug::print::<events::DragEnd>,
                    debug::print::<events::DragEnter>,
                    debug::print::<events::DragOver>.run_if(move || noisy_debug),
                    debug::print::<events::DragLeave>,
                    debug::print::<events::Drop>,
                )
                    .in_set(picking_core::PickSet::Last),
            );

        #[cfg(not(feature = "backend_egui"))]
        app.add_systems(
            PreUpdate,
            (add_pointer_debug, update_debug_data, debug_draw)
                .chain()
                .in_set(picking_core::PickSet::Last),
        );
        #[cfg(feature = "backend_egui")]
        app.add_systems(
            (add_pointer_debug, update_debug_data, debug_draw_egui)
                .chain()
                .in_set(picking_core::PickSet::Last),
        );

        #[cfg(feature = "selection")]
        app.add_systems(
            Update,
            (
                debug::print::<selection::Select>,
                debug::print::<selection::Deselect>,
            ),
        );
    }
}

/// Adds [`PointerDebug`] to pointers automatically.
pub fn add_pointer_debug(
    mut commands: Commands,
    pointers: Query<Entity, (With<PointerId>, Without<PointerDebug>)>,
) {
    for entity in &pointers {
        commands.entity(entity).insert(PointerDebug::default());
    }
}

#[allow(missing_docs)]
#[derive(Debug, Component, Clone, Default)]
pub struct PointerDebug {
    pub location: Option<Location>,
    pub press: PointerPress,
    pub depth: Option<f32>,
    pub world_pos: Option<Vec3>,
    pub world_normal: Option<Vec3>,
    pub drag_start: Vec<(PointerButton, Vec2)>,
    pub interactions: Vec<(DebugName, Interaction)>,
    #[cfg(feature = "selection")]
    pub multiselect: Option<bool>,
}

fn bool_to_icon(f: &mut std::fmt::Formatter, prefix: &str, input: bool) -> std::fmt::Result {
    write!(f, "{prefix}{}", if input { "[X]" } else { "[ ]" })
}

impl std::fmt::Display for PointerDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = &self.location {
            writeln!(f, "Location: {:.2?}", location.position)?;
        }
        bool_to_icon(f, "Pressed: ", self.press.is_primary_pressed())?;
        bool_to_icon(f, " ", self.press.is_middle_pressed())?;
        bool_to_icon(f, " ", self.press.is_secondary_pressed())?;
        #[cfg(feature = "selection")]
        if let Some(multiselect) = self.multiselect {
            bool_to_icon(f, ", Multiselect: ", multiselect)?;
        }
        if let Some((position, normal)) = self.world_pos.zip(self.world_normal) {
            writeln!(f, "\nPosition: {position:.2?} Normal: {normal:.2?}")?;
        }
        if let Some(depth) = self.depth {
            writeln!(f, "Depth: {depth:.2?}")?;
        }
        if !self.interactions.is_empty() {
            write!(f, "{:?}", self.interactions)?;
        }
        Ok(())
    }
}

/// Update typed debug data used to draw overlays
pub fn update_debug_data(
    hover_map: Res<HoverMap>,
    drag_map: Res<DragMap>,
    names: Query<&Name>,
    mut pointers: Query<(
        Entity,
        &pointer::PointerId,
        &pointer::PointerLocation,
        &pointer::PointerPress,
        &focus::PointerInteraction,
        &mut PointerDebug,
    )>,
    #[cfg(feature = "selection")] selection: Query<Option<&selection::PointerMultiselect>>,
) {
    for (entity, id, location, press, interactions, mut debug) in pointers.iter_mut() {
        let interactions = interactions
            .iter()
            .map(|(entity, interaction)| {
                let debug = match names.get(*entity) {
                    Ok(name) => DebugName::Name(name.clone(), *entity),
                    _ => DebugName::Entity(*entity),
                };

                (debug, *interaction)
            })
            .collect::<Vec<_>>();
        let hover_data = |id| hover_map.get(id).and_then(|h| h.iter().next());
        let drag_start = |id| {
            PointerButton::iter()
                .flat_map(|button| {
                    drag_map
                        .get(&(id, button))
                        .and_then(|entry| entry.as_ref())
                        .map(|entry| (button, entry.start_pos))
                })
                .collect()
        };

        *debug = PointerDebug {
            location: location.location().cloned(),
            press: press.to_owned(),
            depth: hover_data(id).map(|h| h.1.depth),
            world_pos: hover_data(id).and_then(|h| h.1.position),
            world_normal: hover_data(id).and_then(|h| h.1.normal),
            drag_start: drag_start(*id),
            interactions,
            #[cfg(feature = "selection")]
            multiselect: selection.get(entity).ok().flatten().map(|f| f.is_pressed),
        };
    }
}

/// Draw an egui window on each cursor with debug info
#[cfg(feature = "backend_egui")]
pub fn debug_draw_egui(
    mut egui: bevy_egui::EguiContexts,
    pointers: Query<(&pointer::PointerId, &PointerDebug)>,
    windows: Query<&Window>,
) {
    use bevy::render::camera::NormalizedRenderTarget;
    use bevy_egui::egui::{self, Color32};

    let transparent_white = Color32::from_rgba_unmultiplied(255, 255, 255, 64);
    let stroke = egui::Stroke::new(3.0, transparent_white);

    for (id, debug) in pointers.iter() {
        let Some(location) = &debug.location else {
            continue
        };
        let NormalizedRenderTarget::Window(window_ref) = location.target else {
            continue;
        };
        let Ok(window) = windows.get(window_ref.entity()) else {
            continue;
        };
        let ctx = egui.ctx_for_window_mut(window_ref.entity());
        let to_egui_pos = |v: Vec2| egui::pos2(v.x, window.height() - v.y);
        let dbg_painter = ctx.layer_painter(egui::LayerId::debug());

        dbg_painter.circle(
            to_egui_pos(location.position),
            20.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 32),
            stroke,
        );

        debug.drag_start.iter().for_each(|(button, drag_start)| {
            let (start, end) = (to_egui_pos(*drag_start), to_egui_pos(location.position));
            dbg_painter.line_segment([start, end], stroke);
            dbg_painter.circle(start, 20.0, egui::Color32::TRANSPARENT, stroke);
            let drag_dist = location.position - *drag_start;
            dbg_painter.debug_text(
                ((end.to_vec2() + start.to_vec2()) * 0.5).to_pos2(),
                egui::Align2::CENTER_CENTER,
                Color32::WHITE,
                format!("{button:?}: [{:.1}, {:.1}]", drag_dist.x, drag_dist.y),
            );
        });

        let text = format!("{id:?} {debug}");
        let alignment = egui::Align2::LEFT_TOP;
        dbg_painter.debug_text(
            (to_egui_pos(location.position).to_vec2()
                - alignment.to_sign() * egui::vec2(20.0, 20.0))
            .to_pos2(),
            alignment,
            egui::Color32::WHITE,
            text,
        );
    }
}

#[allow(missing_docs)]
#[derive(Clone)]
pub enum DebugName {
    Name(Name, Entity),
    Entity(Entity),
}

impl std::fmt::Debug for DebugName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Name(name, entity) => write!(f, "{} ({:?})", name.as_str(), entity),
            Self::Entity(entity) => write!(f, "{entity:?}"),
        }
    }
}

/// Draw text on each cursor with debug info
pub fn debug_draw(
    mut commands: Commands,
    pointers: Query<(Entity, &pointer::PointerId, &PointerDebug)>,
) {
    for (entity, id, debug) in pointers.iter() {
        let Some(location) = &debug.location else {
            continue
        };
        let text = format!("{id:?}\n{debug}");

        commands.entity(entity).insert(TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font: DEBUG_FONT_HANDLE.typed::<Font>(),
                    font_size: 12.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(location.position.x + 5.0),
                top: Val::Px(location.position.y + 5.0),
                ..default()
            },
            ..default()
        });
    }
}
