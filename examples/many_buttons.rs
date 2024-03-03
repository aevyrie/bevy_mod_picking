//! A stress test for picking and events with many interactive elements.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowPlugin},
};
use bevy_mod_picking::prelude::*;

const ROW_COLUMN_COUNT: usize = 110;
const FONT_SIZE: f32 = 7.0;

/// This example shows what happens when there is a lot of buttons on screen.
fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }),
        FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin::default(),
    ))
    .add_plugins(
        DefaultPickingPlugins
            .build()
            .disable::<DebugPickingPlugin>(),
    )
    .insert_resource(DebugPickingMode::Normal)
    .add_systems(Startup, setup)
    .add_systems(Update, update_button_colors);

    if std::env::args().any(|arg| arg == "recompute-layout") {
        app.add_systems(Update, |mut ui_scale: ResMut<UiScale>| {
            ui_scale.set_changed();
        });
    }

    if std::env::args().any(|arg| arg == "recompute-text") {
        app.add_systems(Update, |mut text_query: Query<&mut Text>| {
            text_query
                .iter_mut()
                .for_each(|mut text| text.set_changed());
        });
    }

    app.run();
}

#[derive(Component)]
struct IdleColor(BackgroundColor);

#[allow(clippy::type_complexity)]
/// Use the [`PickingInteraction`] state of each button to update its color.
fn update_button_colors(
    mut buttons: Query<
        (
            Option<&PickingInteraction>,
            &mut BackgroundColor,
            &IdleColor,
        ),
        (With<Button>, Changed<PickingInteraction>),
    >,
) {
    for (interaction, mut button_color, idle_color) in &mut buttons {
        *button_color = match interaction {
            Some(PickingInteraction::Pressed) => Color::rgb(0.35, 0.75, 0.35).into(),
            Some(PickingInteraction::Hovered) => Color::rgb(0.25, 0.25, 0.25).into(),
            Some(PickingInteraction::None) | None => idle_color.0,
        };
    }
}

fn setup(mut commands: Commands) {
    let count = ROW_COLUMN_COUNT;
    let count_f = count as f32;
    let as_rainbow = |i: usize| Color::hsl((i as f32 / count_f) * 360.0, 0.9, 0.8);
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            let spawn_text = std::env::args().all(|arg| arg != "no-text");
            let border = if std::env::args().all(|arg| arg != "no-borders") {
                UiRect::all(Val::Percent(10. / count_f))
            } else {
                UiRect::DEFAULT
            };
            for i in 0..count {
                for j in 0..count {
                    let color = as_rainbow(j % i.max(1)).into();
                    let border_color = as_rainbow(i % j.max(1)).into();
                    spawn_button(
                        commands,
                        color,
                        count_f,
                        i,
                        j,
                        spawn_text,
                        border,
                        border_color,
                    );
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn spawn_button(
    commands: &mut ChildBuilder,
    background_color: BackgroundColor,
    total: f32,
    i: usize,
    j: usize,
    spawn_text: bool,
    border: UiRect,
    border_color: BorderColor,
) {
    let width = 90.0 / total;
    let mut builder = commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Percent(width),
                height: Val::Percent(width),
                bottom: Val::Percent(100.0 / total * i as f32),
                left: Val::Percent(100.0 / total * j as f32),
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                border,
                ..default()
            },
            background_color,
            border_color,
            ..default()
        },
        IdleColor(background_color),
    ));

    if spawn_text {
        builder.with_children(|commands| {
            commands.spawn((
                TextBundle::from_section(
                    format!("{i}, {j}"),
                    TextStyle {
                        font_size: FONT_SIZE,
                        color: Color::rgb(0.2, 0.2, 0.2),
                        ..default()
                    },
                ),
                Pickable::IGNORE,
            ));
        });
    }
}
