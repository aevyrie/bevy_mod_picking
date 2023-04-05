//! Text and on-screen debugging tools

use crate::*;
use bevy::{prelude::*, window::PrimaryWindow};

/// Logs events for debugging
#[derive(Debug, Default, Clone)]
pub struct DebugPickingPlugin {
    /// Suppresses noisy events like `Move` and `Drag` when set to `false`
    pub noisy: bool,
}
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut App) {
        let should_run = self.noisy.into();

        app.init_resource::<core::debug::Frame>()
            .add_system(core::debug::increment_frame.in_base_set(CoreSet::First))
            .add_system(
                input::debug::print
                    .before(core::PickSet::Backend)
                    .run_if(move || should_run)
                    .in_base_set(CoreSet::PreUpdate),
            )
            .add_systems((
                core::debug::print::<output::PointerOver>,
                core::debug::print::<output::PointerOut>,
                core::debug::print::<output::PointerDown>,
                core::debug::print::<output::PointerUp>,
                core::debug::print::<output::PointerClick>,
                core::debug::print::<output::PointerMove>.run_if(move || should_run),
                core::debug::print::<output::PointerDragStart>,
                core::debug::print::<output::PointerDrag>.run_if(move || should_run),
                core::debug::print::<output::PointerDragEnd>,
                core::debug::print::<output::PointerDragEnter>,
                core::debug::print::<output::PointerDragOver>.run_if(move || should_run),
                core::debug::print::<output::PointerDragLeave>,
                core::debug::print::<output::PointerDrop>,
            ));

        #[cfg(not(feature = "backend_egui"))]
        app.add_system(debug_draw);
        #[cfg(feature = "backend_egui")]
        app.add_system(debug_draw_egui);

        #[cfg(feature = "selection")]
        app.add_systems((
            core::debug::print::<selection::PointerSelect>,
            core::debug::print::<selection::PointerDeselect>,
        ));
    }
}

/// Draw an egui window on each cursor with debug info
#[cfg(feature = "backend_egui")]
pub fn debug_draw_egui(
    mut egui: bevy_egui::EguiContexts,
    names: Query<&Name>,
    pointers: Query<(
        Entity,
        &pointer::PointerId,
        &pointer::PointerLocation,
        &pointer::PointerPress,
        &output::PointerInteraction,
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

        let text = format!("ID: {:?}\nLocation: x{} y{}\nPress Primary: {}, Secondary: {}, Middle: {}\n{}Interactions: {:?}",
                id,
                position.x,
                position.y,
                bool_to_icon(&press.is_primary_pressed()),
                bool_to_icon(&press.is_secondary_pressed()),
                bool_to_icon(&press.is_middle_pressed()),
                selection,
                interaction,
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
    asset_server: Res<AssetServer>,
    pointers: Query<(
        Entity,
        &pointer::PointerId,
        &pointer::PointerLocation,
        &pointer::PointerPress,
        &output::PointerInteraction,
    )>,
    #[cfg(feature = "selection")] selection: Query<Option<&selection::PointerMultiselect>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    for (entity, id, location, press, interaction) in pointers.iter() {
        let location = match location.location() {
            Some(l) => l.position,
            None => continue,
        };
        let x = primary_window.single().width() - location.x;
        let y = primary_window.single().height() - location.y;

        let mut text = Text::from_section(
            ".\n",
            TextStyle {
                font: font.clone(),
                font_size: 42.0,
                color: Color::RED,
            },
        );

        #[cfg(feature = "selection")]
        let selection = selection
            .get(entity)
            .ok()
            .flatten()
            .map(|f| format!("Multiselect: {}\n", f.is_pressed))
            .unwrap_or_else(|| String::from("Multiselect: pointer disabled\n"));
        #[cfg(not(feature = "selection"))]
        let selection = String::new();

        text.sections.push(TextSection::new(
            format!("ID: {:?}\nLocation: x{} y{}\nPress (Primary, Secondary, Middle): ({}, {}, {})\n{}Interactions: {:?}\n",
                id,
                location.x,
                location.y,
                press.is_primary_pressed(),
                press.is_secondary_pressed(),
                press.is_middle_pressed(),
                selection,
                interaction.iter()
            ),
            TextStyle {
                font: font.clone(),
                font_size: 12.0,
                color: Color::WHITE,
            },
        ));
        text.alignment = TextAlignment::Right;

        commands.entity(entity).insert(TextBundle {
            text,
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(x - 8.0),
                    top: Val::Px(y - 31.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        });
    }
}
