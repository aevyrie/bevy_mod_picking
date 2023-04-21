use std::marker::PhantomData;

use bevy::{ecs::system::Command, prelude::*, ui::FocusPolicy};
use bevy_mod_picking::prelude::*;

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
            OnPointer::<Over>::add_command::<SetColor<Hovered>>(),
            OnPointer::<Out>::add_command::<SetColor<Normal>>(),
            OnPointer::<Down>::add_command::<SetColor<Pressed>>(),
            OnPointer::<Up>::add_command::<SetColor<Normal>>(),
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

trait AsColor: Send + 'static {
    const COLOR: Color;
}

struct Normal;
impl AsColor for Normal {
    const COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
}

struct Hovered;
impl AsColor for Hovered {
    const COLOR: Color = Color::rgb(0.25, 0.25, 0.25);
}

struct Pressed;
impl AsColor for Pressed {
    const COLOR: Color = Color::rgb(0.35, 0.75, 0.35);
}

struct SetColor<C: AsColor>(Entity, PhantomData<C>);
impl<C: AsColor> Command for SetColor<C> {
    fn write(self, world: &mut World) {
        world
            .entity_mut(self.0)
            .insert(BackgroundColor::from(C::COLOR));
    }
}

impl<E: IsPointerEvent, C: AsColor> From<ListenedEvent<E>> for SetColor<C> {
    fn from(event: ListenedEvent<E>) -> Self {
        SetColor(event.target, PhantomData)
    }
}
