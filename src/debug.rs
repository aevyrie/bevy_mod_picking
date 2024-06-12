//! Text and on-screen debugging tools

use std::fmt::Debug;

use bevy_core::Name;
use bevy_picking_core::focus::HoverMap;
use picking_core::{backend::HitData, events::DragMap, pointer::Location};

use crate::*;

use bevy_app::prelude::*;
use bevy_math::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::prelude::*;
use bevy_utils::tracing::{debug, trace};

/// This resource determines the runtime behavior of the debug plugin.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Resource)]
pub enum DebugPickingMode {
    /// Only log non-noisy events, show the debug overlay.
    Normal,
    /// Log all events, including noisy events like `Move` and `Drag`, show the debug overlay.
    Noisy,
    /// Do not show the debug overlay or log any messages.
    #[default]
    Disabled,
}

impl DebugPickingMode {
    /// A condition indicating the plugin is enabled
    pub fn is_enabled(this: Res<Self>) -> bool {
        matches!(*this, Self::Normal | Self::Noisy)
    }
    /// A condition indicating the plugin is disabled
    pub fn is_disabled(this: Res<Self>) -> bool {
        matches!(*this, Self::Disabled)
    }
    /// A condition indicating the plugin is enabled and in noisy mode
    pub fn is_noisy(this: Res<Self>) -> bool {
        matches!(*this, Self::Noisy)
    }
}

/// Logs events for debugging
///
/// "Normal" events are logged at the `debug` level. "Noisy" events are logged at the `trace` level.
/// See [Bevy's LogPlugin](https://docs.rs/bevy/latest/bevy/log/struct.LogPlugin.html) and [Bevy
/// Cheatbook: Logging, Console Messages](https://bevy-cheatbook.github.io/features/log.html) for
/// details.
///
/// Usually, the default level printed is `info`, so debug and trace messages will not be displayed
/// even when this plugin is active. You can set `RUST_LOG` to change this. For example:
///
/// ```bash
/// RUST_LOG="warn,bevy_mod_picking=trace,bevy_ui=info" cargo run --example bevy_ui
/// ```
///
/// You can also change the log filter at runtime in your code. The [LogPlugin
/// docs](https://docs.rs/bevy/latest/bevy/log/struct.LogPlugin.html) give an example.
///
/// Use the [`DebugPickingMode`] state resource to control this plugin. Example:
///
/// ```ignore
/// use DebugPickingMode::{Normal, Disabled};
/// app.add_plugin(DefaultPickingPlugins)
///     .insert_resource(DebugPickingMode::Normal)
///     .add_systems(
///         PreUpdate,
///         (|mut mode: ResMut<DebugPickingMode>| {
///             *mode = match *mode {
///                 DebugPickingMode::Disabled => DebugPickingMode::Normal,
///                 _ => DebugPickingMode::Disabled,
///             };
///         })
///         .distributive_run_if(bevy::input::common_conditions::input_just_pressed(
///             KeyCode::F3,
///         )),
///     )
/// ```
/// This sets the starting mode of the plugin to [`DebugPickingMode::Disabled`] and binds the F3 key
/// to toggle it.
#[derive(Debug, Default, Clone)]
pub struct DebugPickingPlugin;

impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugPickingMode>()
            .add_systems(
                PreUpdate,
                pointer_debug_visibility.in_set(picking_core::PickSet::PostFocus),
            )
            .add_systems(
                PreUpdate,
                (
                    // This leaves room to easily change the log-level associated
                    // with different events, should that be desired.
                    log_event_debug::<pointer::InputMove>.run_if(DebugPickingMode::is_noisy),
                    log_event_debug::<pointer::InputPress>.run_if(DebugPickingMode::is_noisy),
                    log_pointer_event_debug::<events::Over>,
                    log_pointer_event_debug::<events::Out>,
                    log_pointer_event_debug::<events::Down>,
                    log_pointer_event_debug::<events::Up>,
                    log_pointer_event_debug::<events::Click>,
                    log_pointer_event_trace::<events::Move>.run_if(DebugPickingMode::is_noisy),
                    log_pointer_event_debug::<events::DragStart>,
                    log_pointer_event_trace::<events::Drag>.run_if(DebugPickingMode::is_noisy),
                    log_pointer_event_debug::<events::DragEnd>,
                    log_pointer_event_debug::<events::DragEnter>,
                    log_pointer_event_trace::<events::DragOver>.run_if(DebugPickingMode::is_noisy),
                    log_pointer_event_debug::<events::DragLeave>,
                    log_pointer_event_debug::<events::Drop>,
                )
                    .distributive_run_if(DebugPickingMode::is_enabled)
                    .in_set(picking_core::PickSet::Last),
            );

        app.add_systems(
            PreUpdate,
            (
                add_pointer_debug,
                update_debug_data,
                // if bevy ui is available, and egui is not, we just use the bevy ui debug draw
                #[cfg(all(feature = "backend_bevy_ui", not(feature = "backend_egui")))]
                debug_draw,
                // if both are available, we only run the bevy ui one while egui is not set up
                #[cfg(all(feature = "backend_bevy_ui", feature = "backend_egui"))]
                debug_draw.run_if(|r: Option<Res<bevy_egui::EguiUserTextures>>| r.is_none()),
                // if egui is available, always draw the egui debug if possible
                #[cfg(feature = "backend_egui")]
                debug_draw_egui.run_if(|r: Option<Res<bevy_egui::EguiUserTextures>>| r.is_some()),
            )
                .chain()
                .distributive_run_if(DebugPickingMode::is_enabled)
                .in_set(picking_core::PickSet::Last),
        );

        #[cfg(feature = "selection")]
        app.add_systems(
            Update,
            (
                debug::log_pointer_event_debug::<selection::Select>,
                debug::log_pointer_event_debug::<selection::Deselect>,
            )
                .distributive_run_if(DebugPickingMode::is_enabled),
        );
    }
}

/// Listen for any event and logs it at the debug level
pub fn log_event_debug<E: Event + Debug>(mut events: EventReader<pointer::InputMove>) {
    for event in events.read() {
        debug!("{event:?}");
    }
}

/// Listens for pointer events of type `E` and logs them at "debug" level
pub fn log_pointer_event_debug<E: Debug + Clone + Reflect>(
    mut pointer_events: EventReader<Pointer<E>>,
) {
    for event in pointer_events.read() {
        debug!("{event}");
    }
}

/// Listens for pointer events of type `E` and logs them at "trace" level
pub fn log_pointer_event_trace<E: Debug + Clone + Reflect>(
    mut pointer_events: EventReader<Pointer<E>>,
) {
    for event in pointer_events.read() {
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
pub fn pointer_debug_visibility(
    debug: Res<DebugPickingMode>,
    mut pointers: Query<&mut Visibility, With<PointerId>>,
) {
    let visible = match *debug {
        DebugPickingMode::Disabled => Visibility::Hidden,
        _ => Visibility::Visible,
    };
    for mut vis in &mut pointers {
        *vis = visible;
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
    use bevy_egui::egui::{self, Color32};
    use bevy_render::camera::NormalizedRenderTarget;

    let transparent_white = Color32::from_rgba_unmultiplied(255, 255, 255, 64);
    let stroke = egui::Stroke::new(3.0, transparent_white);

    for (id, debug) in pointers.iter() {
        let Some(location) = &debug.location else {
            continue;
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

#[cfg(feature = "backend_bevy_ui")]
/// Draw text on each cursor with debug info
pub fn debug_draw(
    mut commands: Commands,
    camera_query: Query<(Entity, &Camera)>,
    primary_window: Query<Entity, With<bevy_window::PrimaryWindow>>,
    pointers: Query<(Entity, &pointer::PointerId, &PointerDebug)>,
    scale: Res<bevy_ui::UiScale>,
) {
    use bevy_text::prelude::*;
    use bevy_ui::prelude::*;
    for (entity, id, debug) in pointers.iter() {
        let Some(pointer_location) = &debug.location else {
            continue;
        };
        let text = format!("{id:?}\n{debug}");

        for camera in camera_query
            .iter()
            .map(|(entity, camera)| {
                (
                    entity,
                    camera.target.normalize(primary_window.get_single().ok()),
                )
            })
            .filter_map(|(entity, target)| Some(entity).zip(target))
            .filter(|(_entity, target)| target == &pointer_location.target)
            .map(|(cam_entity, _target)| cam_entity)
        {
            let mut pointer_pos = pointer_location.position;
            if let Some(viewport) = camera_query
                .get(camera)
                .ok()
                .and_then(|(_, camera)| camera.logical_viewport_rect())
            {
                pointer_pos -= viewport.min;
            }

            commands
                .entity(entity)
                .insert(TextBundle {
                    text: Text::from_section(
                        text.clone(),
                        TextStyle {
                            font_size: 12.0,
                            color: bevy_color::Color::WHITE,
                            ..Default::default()
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(pointer_pos.x + 5.0) / scale.0,
                        top: Val::Px(pointer_pos.y + 5.0) / scale.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Pickable::IGNORE)
                .insert(TargetCamera(camera));
        }
    }
}
