//! This example demonstrates how to use the plugin with bevy_ui.

use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::prelude::*;

const NORMAL: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED: Color = Color::rgb(0.35, 0.75, 0.35);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup);
    #[cfg(feature = "backend_egui")]
    app.add_plugin(bevy_egui::EguiPlugin);
    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                    margin: UiRect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                ..default()
            },
            OnPointer::<Over>::target_insert(BackgroundColor::from(HOVERED)),
            OnPointer::<Out>::target_insert(BackgroundColor::from(NORMAL)),
            OnPointer::<Down>::target_insert(BackgroundColor::from(PRESSED)),
            OnPointer::<Up>::target_insert(BackgroundColor::from(HOVERED)),
            OnPointer::<Drag>::target_component_mut::<Style>(|drag, style| {
                style.size.width.try_add_assign(Val::Px(drag.delta.x)).ok();
                style.size.height.try_add_assign(Val::Px(drag.delta.y)).ok();
            }),
        ))
        .with_children(|parent| {
            let mut bundle = TextBundle::from_section(
                "Drag Me!",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            );
            bundle.focus_policy = FocusPolicy::Pass;
            parent.spawn(bundle);
        });
}
