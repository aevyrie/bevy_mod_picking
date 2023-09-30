//! Text and on-screen debugging tools

use std::fmt::Debug;

use bevy_picking_core::focus::HoverMap;
use picking_core::{backend::HitData, events::DragMap, pointer::Location};

use crate::*;
use bevy::prelude::*;

/// This state determines the runtime behavior of the debug plugin.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum DebugPickingMode {
    /// Only log non-noisy events
    #[default]
    Normal,
    /// Log all events, including noisy events like `Move` and `Drag`
    Noisy,
    /// Do not show the debug interface or log any messages
    Disabled,
}

impl DebugPickingMode {
    /// A condition indicating the plugin is enabled
    pub fn is_enabled(res: Res<State<DebugPickingMode>>) -> bool {
        matches!(res.get(), Self::Normal | Self::Noisy)
    }
    /// A condition indicating the plugin is disabled
    pub fn is_disabled(res: Res<State<DebugPickingMode>>) -> bool {
        matches!(res.get(), Self::Disabled)
    }
    /// A condition indicating the plugin is enabled and in noisy mode
    pub fn is_noisy(res: Res<State<DebugPickingMode>>) -> bool {
        matches!(res.get(), Self::Noisy)
    }
}

/// Logs events for debugging
///
/// "Normal" events are logged at the `debug` level.
/// "Noisy" events are logged at the `trace` level.
/// See [Bevy's LogPlugin](https://docs.rs/bevy/latest/bevy/log/struct.LogPlugin.html)
/// and [Bevy Cheatbook: Logging, Console Messages](https://bevy-cheatbook.github.io/features/log.html)
/// for details.
///
/// Usually, the default level printed is `info`, so debug and trace messages will
/// not be displayed even when this plugin is active. You can set `RUST_LOG` to
/// change this. For example:
///
/// ```bash
/// RUST_LOG="warn,bevy_mod_picking=trace,bevy_ui=info" cargo run --example bevy_ui
/// ```
///
/// You can also change the log filter at runtime in your code.
/// The [LogPlugin docs](https://docs.rs/bevy/latest/bevy/log/struct.LogPlugin.html)
/// give an example.
///
/// Use the [`DebugPickingMode`] state resource to control this plugin.
/// Example:
///
/// ```ignore
/// use DebugPickingMode::{Normal, Disabled};
/// app.add_plugin(DefaultPickingPlugins)
///     .insert_resource(State(Disabled))
///     .add_systems(
///         (
///             (|mut next: ResMut<NextState<_>>| next.set(Normal)).run_if(in_state(Disabled)),
///             (|mut next: ResMut<NextState<_>>| next.set(Disabled)).run_if(in_state(Normal)),
///         )
///             .distributive_run_if(input_just_pressed(KeyCode::F3)),
///     )
/// ```
/// This sets the starting mode of the plugin to [`DebugPickingMode::Disabled`] and binds
/// the F3 key to toggle it.
#[derive(Debug, Default, Clone)]
pub struct DebugPickingPlugin {
    /// Suppresses noisy events like `Move` and `Drag` when set to `false`.
    #[deprecated = "Use the `DebugPickingMode` state directly instead"]
    pub noisy: bool,
}

impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut App) {
        #[allow(deprecated)]
        let start_mode = if self.noisy {
            DebugPickingMode::Noisy
        } else {
            DebugPickingMode::Normal
        };

        app.add_state::<DebugPickingMode>()
            .insert_resource(State::new(start_mode))
            .add_systems(
                PreUpdate,
                input::debug::print
                    .before(picking_core::PickSet::Backend)
                    .run_if(DebugPickingMode::is_noisy),
            );
        app.add_systems(
            PreUpdate,
            (
                // This leaves room to easily change the log-level associated
                // with different events, should that be desired.
                log_debug::<events::Over>,
                log_debug::<events::Out>,
                log_debug::<events::Down>,
                log_debug::<events::Up>,
                log_debug::<events::Click>,
                log_trace::<events::Move>.run_if(DebugPickingMode::is_noisy),
                log_debug::<events::DragStart>,
                log_trace::<events::Drag>.run_if(DebugPickingMode::is_noisy),
                log_debug::<events::DragEnd>,
                log_debug::<events::DragEnter>,
                log_trace::<events::DragOver>.run_if(DebugPickingMode::is_noisy),
                log_debug::<events::DragLeave>,
                log_debug::<events::Drop>,
            )
                .distributive_run_if(DebugPickingMode::is_enabled)
                .in_set(picking_core::PickSet::Last),
        );

        #[cfg(not(feature = "backend_egui"))]
        app.add_systems(
            PreUpdate,
            (add_pointer_debug, update_debug_data, debug_draw)
                .chain()
                .distributive_run_if(DebugPickingMode::is_enabled)
                .in_set(picking_core::PickSet::Last),
        );
        #[cfg(feature = "backend_egui")]
        app.add_systems(
            PreUpdate,
            (
                add_pointer_debug,
                update_debug_data,
                debug_draw.run_if(|r: Option<Res<bevy_egui::EguiUserTextures>>| r.is_none()),
                debug_draw_egui.run_if(|r: Option<Res<bevy_egui::EguiUserTextures>>| r.is_some()),
            )
                .chain()
                .distributive_run_if(DebugPickingMode::is_enabled)
                .in_set(picking_core::PickSet::Last),
        );

        app.add_systems(OnEnter(DebugPickingMode::Disabled), hide_pointer_text);

        #[cfg(feature = "selection")]
        app.add_systems(
            Update,
            (
                debug::log_debug::<selection::Select>,
                debug::log_debug::<selection::Deselect>,
            )
                .distributive_run_if(DebugPickingMode::is_enabled),
        );
    }
}

/// Listens for pointer events of type `E` and logs them at "debug" level
pub fn log_debug<E: Debug + Clone + Reflect>(mut pointer_events: EventReader<Pointer<E>>) {
    for event in pointer_events.iter() {
        debug!("{event}");
    }
}

/// Listens for pointer events of type `E` and logs them at "trace" level
pub fn log_trace<E: Debug + Clone + Reflect>(mut pointer_events: EventReader<Pointer<E>>) {
    for event in pointer_events.iter() {
        trace!("{event}");
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

/// Hide text from pointers.
pub fn hide_pointer_text(mut pointers: Query<&mut Visibility, With<PointerId>>) {
    for mut vis in &mut pointers {
        *vis = Visibility::Hidden;
    }
}

#[allow(missing_docs)]
#[derive(Debug, Component, Clone, Default)]
pub struct PointerDebug {
    pub location: Option<Location>,
    pub press: PointerPress,
    pub hits: Vec<(DebugName, HitData)>,
    pub drag_start: Vec<(PointerButton, Vec2)>,
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
        let mut sorted_hits = self.hits.clone();
        sorted_hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (entity, hit) in sorted_hits.iter() {
            write!(f, "\nEntity: {entity:?}")?;
            if let Some((position, normal)) = hit.position.zip(hit.normal) {
                write!(f, ", Position: {position:.2?}, Normal: {normal:.2?}")?;
            }
            write!(f, ", Depth: {:.2?}", hit.depth)?;
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
        &mut PointerDebug,
    )>,
    #[cfg(feature = "selection")] selection: Query<Option<&selection::PointerMultiselect>>,
) {
    for (entity, id, location, press, mut debug) in pointers.iter_mut() {
        let drag_start = |id| {
            PointerButton::iter()
                .flat_map(|button| {
                    drag_map
                        .get(&(id, button))
                        .and_then(|entry| entry.values().next())
                        .map(|entry| (button, entry.start_pos))
                })
                .collect()
        };

        *debug = PointerDebug {
            location: location.location().cloned(),
            press: press.to_owned(),
            hits: hover_map
                .get(id)
                .iter()
                .flat_map(|h| h.iter())
                .map(|(e, h)| {
                    (
                        if let Ok(name) = names.get(*e) {
                            DebugName::Name(name.clone(), *e)
                        } else {
                            DebugName::Entity(*e)
                        },
                        h.to_owned(),
                    )
                })
                .collect(),
            drag_start: drag_start(*id),
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
        let ctx = egui.ctx_for_window_mut(window_ref.entity());
        let to_egui_pos = |v: Vec2| egui::pos2(v.x, v.y);
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
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum DebugName {
    Name(Name, Entity),
    Entity(Entity),
}

impl Debug for DebugName {
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

        commands
            .entity(entity)
            .insert(TextBundle {
                text: Text::from_section(
                    text,
                    TextStyle {
                        font_size: 12.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(location.position.x + 5.0),
                    top: Val::Px(location.position.y + 5.0),
                    ..default()
                },
                ..default()
            })
            .insert(Pickable::IGNORE);
    }
}
