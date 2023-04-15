//! Text and on-screen debugging tools

use bevy_picking_core::{debug, focus::HoverMap};

use crate::*;
use bevy::{asset::load_internal_binary_asset, prelude::*, utils::Uuid, window::PrimaryWindow};

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
            .add_system(debug::increment_frame.in_base_set(CoreSet::First))
            .add_system(
                input::debug::print
                    .before(core::PickSet::Backend)
                    .run_if(move || noisy_debug)
                    .in_base_set(CoreSet::PreUpdate),
            )
            .add_systems((
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
            ));

        #[cfg(not(feature = "backend_egui"))]
        app.add_system(debug_draw);
        #[cfg(feature = "backend_egui")]
        app.add_system(debug_draw_egui);

        #[cfg(feature = "selection")]
        app.add_systems((
            debug::print::<selection::Select>,
            debug::print::<selection::Deselect>,
        ));
    }
}

/// Draw an egui window on each cursor with debug info
#[cfg(feature = "backend_egui")]
pub fn debug_draw_egui(
    mut egui: bevy_egui::EguiContexts,
    hover_map: Res<HoverMap>,
    names: Query<&Name>,
    pointers: Query<(
        Entity,
        &pointer::PointerId,
        &pointer::PointerLocation,
        &pointer::PointerPress,
        &events::PointerInteraction,
    )>,
    #[cfg(feature = "selection")] selection: Query<Option<&selection::PointerMultiselect>>,
    windows: Query<&Window>,
    mut alignment: Local<Option<bevy_egui::egui::Align2>>,
) {
    use bevy::render::camera::NormalizedRenderTarget;

    for (entity, id, location, press, interaction) in pointers.iter() {
        let location = match location.location() {
            Some(l) => l,
            None => continue,
        };
        let position = location.position;

        let NormalizedRenderTarget::Window(window_ref) = location.target else {
            continue;
        };

        let ctx = egui.ctx_for_window_mut(window_ref.entity());

        let Ok(window) = windows.get(window_ref.entity()) else {
            continue;
        };

        let x = position.x;
        let y = window.height() - position.y;

        let left = x > window.width() / 2.0;
        let top = y > window.height() / 2.0;

        let near_border =
            window.width() - x < 300.0 || x < 300.0 || window.height() - y < 150.0 || y < 150.0;

        fn bool_to_icon(from: &bool) -> &str {
            if *from {
                "☑"
            } else {
                "☐"
            }
        }

        #[cfg(feature = "selection")]
        let selection = selection
            .get(entity)
            .ok()
            .flatten()
            .map(|f| format!("Multiselect: {}\n", bool_to_icon(&f.is_pressed)))
            .unwrap_or_else(|| String::from("Multiselect: pointer disabled\n"));
        #[cfg(not(feature = "selection"))]
        let selection = String::new();

        let interaction = interaction
            .iter()
            .map(|(entity, interaction)| {
                let debug = match names.get(*entity) {
                    Ok(name) => InteractionDebug::Name(name.clone(), *entity),
                    _ => InteractionDebug::Entity(*entity),
                };

                (debug, interaction)
            })
            .collect::<Vec<_>>();

        let hover_data = |id| hover_map.get(id).and_then(|h| h.iter().next());

        let text = format!("ID: {:?}\nLocation: x{} y{}\nPress (Primary {}, Secondary {}, Middle {})\n{}Depth: {:0.2?}\nPosition: {:0.2?}\nNormal: {:0.2?}\nInteractions: {:?}\n",
            id,
            position.x,
            position.y,
            bool_to_icon(&press.is_primary_pressed()),
            bool_to_icon(&press.is_secondary_pressed()),
            bool_to_icon(&press.is_middle_pressed()),
            selection,
            hover_data(id).map(|h| h.1.depth),
            hover_data(id).and_then(|h| h.1.position),
            hover_data(id).and_then(|h| h.1.normal),
            interaction.iter()
        );
        use bevy_egui::egui;

        let center = egui::pos2(x, y);

        let dbg_painter = ctx.layer_painter(egui::LayerId::debug());
        dbg_painter.circle(
            center,
            20.0,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(
                3.0,
                if press.is_any_pressed() {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                },
            ),
        );

        let new_alignment = match (left, top) {
            (true, true) => egui::Align2::RIGHT_BOTTOM,
            (true, false) => egui::Align2::RIGHT_TOP,
            (false, true) => egui::Align2::LEFT_BOTTOM,
            (false, false) => egui::Align2::LEFT_TOP,
        };

        dbg_painter.debug_text(
            (center.to_vec2() - new_alignment.to_sign() * egui::vec2(20.0, 20.0)).to_pos2(),
            match (alignment.to_owned(), near_border) {
                (Some(cached), false) => cached,
                _ => {
                    *alignment = Some(new_alignment);
                    new_alignment
                }
            },
            egui::Color32::WHITE,
            text,
        );
    }
}

#[cfg(feature = "backend_egui")]
enum InteractionDebug {
    Name(Name, Entity),
    Entity(Entity),
}

#[cfg(feature = "backend_egui")]
impl std::fmt::Debug for InteractionDebug {
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
    hover_map: Res<HoverMap>,
    pointers: Query<(
        Entity,
        &pointer::PointerId,
        &pointer::PointerLocation,
        &pointer::PointerPress,
        &focus::PointerInteraction,
    )>,
    #[cfg(feature = "selection")] selection: Query<Option<&selection::PointerMultiselect>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    for (entity, id, location, press, interaction) in pointers.iter() {
        let location = match location.location() {
            Some(l) => l.position,
            None => continue,
        };
        let x = primary_window.single().width() - location.x;
        let y = primary_window.single().height() - location.y;
        let hover_data = |id| hover_map.get(id).and_then(|h| h.iter().next());

        #[cfg(feature = "selection")]
        let selection = selection
            .get(entity)
            .ok()
            .flatten()
            .map(|f| format!("Multiselect: {}\n", f.is_pressed))
            .unwrap_or_else(|| String::from("Multiselect: pointer disabled\n"));
        #[cfg(not(feature = "selection"))]
        let selection = String::new();

        let mut text = Text::from_section(
            format!("ID: {:?}\nLocation: x{} y{}\nPress (Primary, Secondary, Middle): ({}, {}, {})\n{}Depth: {:0.2?}\nPosition: {:0.2?}\nNormal: {:0.2?}\nInteractions: {:?}\n",
                id,
                location.x,
                location.y,
                press.is_primary_pressed(),
                press.is_secondary_pressed(),
                press.is_middle_pressed(),
                selection,
                hover_data(id).map(|h| h.1.depth),
                hover_data(id).and_then(|h| h.1.position),
                hover_data(id).and_then(|h| h.1.normal),
                interaction.iter()
            ),
            TextStyle {
                font: DEBUG_FONT_HANDLE.typed::<Font>(),
                font_size: 12.0,
                color: Color::WHITE,
            },
        );
        text.alignment = TextAlignment::Left;

        commands.entity(entity).insert(TextBundle {
            text,
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(x - 345.0),
                    top: Val::Px(y - 110.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        });
    }
}
