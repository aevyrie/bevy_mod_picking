//! Text and on-screen debugging tools

use crate::*;
use bevy::prelude::*;

/// Logs events for debugging
#[derive(Debug, Default, Clone)]
pub struct DebugPickingPlugin {
    /// Suppresses noisy events like `Move` and `Drag` when set to `false`
    pub noisy: bool,
}
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut App) {
        let should_run = if self.noisy {
            bevy::ecs::schedule::ShouldRun::Yes
        } else {
            bevy::ecs::schedule::ShouldRun::No
        };

        app.init_resource::<core::debug::Frame>()
            .add_system_to_stage(CoreStage::First, core::debug::increment_frame)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                input::debug::print
                    .before(core::PickStage::Backend)
                    .with_run_criteria(move || should_run),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_system(core::debug::print::<output::PointerOver>)
                    .with_system(core::debug::print::<output::PointerOut>)
                    .with_system(core::debug::print::<output::PointerDown>)
                    .with_system(core::debug::print::<output::PointerUp>)
                    .with_system(core::debug::print::<output::PointerClick>)
                    .with_system(
                        core::debug::print::<output::PointerMove>
                            .with_run_criteria(move || should_run),
                    )
                    .with_system(core::debug::print::<output::PointerDragStart>)
                    .with_system(
                        core::debug::print::<output::PointerDrag>
                            .with_run_criteria(move || should_run),
                    )
                    .with_system(core::debug::print::<output::PointerDragEnd>)
                    .with_system(core::debug::print::<output::PointerDragEnter>)
                    .with_system(
                        core::debug::print::<output::PointerDragOver>
                            .with_run_criteria(move || should_run),
                    )
                    .with_system(core::debug::print::<output::PointerDragLeave>)
                    .with_system(core::debug::print::<output::PointerDrop>)
                    .label("PointerOutputDebug"),
            )
            .add_system(debug_draw);

        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .with_system(core::debug::print::<selection::PointerSelect>)
                .with_system(core::debug::print::<selection::PointerDeselect>),
        );
    }
}

fn debug_draw(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pointers: Query<(
        Entity,
        &pointer::PointerId,
        &pointer::PointerLocation,
        &pointer::PointerPress,
        &output::PointerInteraction,
        Option<&selection::PointerMultiselect>,
    )>,
    windows: Res<Windows>,
) {
    for (entity, id, location, press, interaction, selection) in pointers.iter() {
        let location = match location.location() {
            Some(l) => l.position,
            None => continue,
        };
        let x = windows.primary().width() - location.x;
        let y = windows.primary().height() - location.y;

        let font = asset_server.load("fonts/FiraMono-Medium.ttf");

        let mut text = Text::from_section(
            ".\n",
            TextStyle {
                font: font.clone(),
                font_size: 42.0,
                color: Color::RED,
            },
        );
        text.sections.push(TextSection::new(
            format!("ID: {:?}\nLocation: x{} y{}\nPress (Primary, Secondary, Middle): ({}, {}, {})\nMultiselect: {}\nInteractions: {:?}\n",
                id,
                location.x,
                location.y,
                press.is_primary_pressed(),
                press.is_secondary_pressed(),
                press.is_middle_pressed(),
                selection.map (|f| format!("{}", f.is_pressed)).unwrap_or_else(|| String::from("disabled")),
                interaction.iter()
            ),
            TextStyle {
                font,
                font_size: 12.0,
                color: Color::WHITE,
            },
        ));
        text.alignment = TextAlignment::TOP_RIGHT;

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
