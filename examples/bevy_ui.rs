use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::prelude::*;

const NORMAL: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED: Color = Color::rgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                    margin: UiRect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                ..default()
            },
            OnPointer::<Over>::insert_on_target(BackgroundColor::from(HOVERED)),
            OnPointer::<Out>::insert_on_target(BackgroundColor::from(NORMAL)),
            OnPointer::<Down>::insert_on_target(BackgroundColor::from(PRESSED)),
            OnPointer::<Up>::insert_on_target(BackgroundColor::from(NORMAL)),
        ))
        .with_children(|parent| {
            let mut bundle = TextBundle::from_section(
                "Button",
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
