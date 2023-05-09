//! This example demonstrates how to use the plugin with bevy_ui.

use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
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
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands.spawn(Camera2dBundle::default());
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Px(500.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                margin: UiRect::horizontal(Val::Auto),
                ..default()
            },
            ..default()
        })
        .id();

    commands
        .entity(root)
        .add_button(&font, "Start")
        .add_button(&font, "Settings")
        .add_button(&font, "Quit");
}

trait NewButton {
    fn add_button(self, font: &Handle<Font>, text: &str) -> Self;
}

impl<'w, 's, 'a> NewButton for EntityCommands<'w, 's, 'a> {
    fn add_button(mut self, font: &Handle<Font>, text: &str) -> Self {
        let child = self
            .commands()
            .spawn((
                ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(42.0)),
                        margin: UiRect::top(Val::Percent(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL.into(),
                    focus_policy: FocusPolicy::Block,
                    ..default()
                },
                // Use events to highlight buttons
                OnPointer::<Over>::listener_insert(BackgroundColor::from(HOVERED)),
                OnPointer::<Out>::listener_insert(BackgroundColor::from(NORMAL)),
                OnPointer::<Down>::listener_insert(BackgroundColor::from(PRESSED)),
                OnPointer::<Up>::listener_insert(BackgroundColor::from(HOVERED)),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        text,
                        TextStyle {
                            font: font.to_owned(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    // If we don't block, the event will be sent twice, because picking the text
                    // will pass through and pick the button container as well.
                    focus_policy: FocusPolicy::Block,
                    ..Default::default()
                });
            })
            .id();
        self.add_child(child);
        self
    }
}
