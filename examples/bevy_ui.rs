use bevy::{
    ecs::system::{Command, EntityCommand},
    prelude::*,
    ui::FocusPolicy,
};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(setup)
        .run();
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
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
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            EventListener::<Over>::callback(|commands, event, _| {
                commands
                    .entity(event.target)
                    .insert(BackgroundColor::from(HOVERED_BUTTON));
            }),
            EventListener::<Out>::callback(|commands, event, _| {
                commands
                    .entity(event.target)
                    .insert(BackgroundColor::from(NORMAL_BUTTON));
            }),
            EventListener::<Down>::callback(|commands, event, _| {
                commands
                    .entity(event.target)
                    .insert(BackgroundColor::from(PRESSED_BUTTON));
            }),
            EventListener::<Up>::callback(|commands, event, _| {
                commands
                    .entity(event.target)
                    .insert(BackgroundColor::from(NORMAL_BUTTON));
            }),
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

struct SetTargetBg(Entity, Color);
impl Command for SetTargetBg {
    fn write(self, world: &mut World) {
        world
            .entity_mut(self.0)
            .insert(BackgroundColor::from(self.1));
    }
}

fn set_background_color<E: IsPointerEvent>(
    color: Color,
) -> fn(&mut Commands, &ListenedEvent<E>, &mut Bubble) {
    |commands, event, _| {
        commands
            .entity(event.target)
            .insert(BackgroundColor::from(NORMAL_BUTTON));
    }
}
