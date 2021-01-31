use bevy::render::camera::PerspectiveProjection;
use bevy::{prelude::*, render::camera::Camera};
use bevy_mod_picking::*; // Import all the picking goodies!

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            vsync: false, // Disabled for this demo to remove vsync as a source of input latency
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        // PickingPlugin provides core picking systems and must be registered first
        .add_plugin(PickingPlugin)
        // InteractablePickingPlugin adds mouse events and selection
        .add_plugin(InteractablePickingPlugin)
        // HighlightablePickingPlugin adds hover, click, and selection highlighting
        .add_plugin(HighlightablePickingPlugin)
        // DebugPickingPlugin systems to build and update debug cursors
        .add_plugin(DebugCursorPickingPlugin)
        // Debug output for interaction events
        .add_plugin(DebugEventsPickingPlugin)
        .add_startup_system(setup.system())
        .init_resource::<ButtonMaterials>()
        .add_startup_system(setup_ui.system())
        .add_system(button_system.system())
        .add_system(oscillation_system.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    // camera
    commands
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(-3.0, 5.0, 8.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with_bundle(PickingCameraBundle::default())
        //plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        // sphere
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 20,
                radius: 0.5,
            })),
            transform: Transform::from_translation(Vec3::new(1.5, 1.5, 1.5)),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Mutated<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *material = button_materials.pressed.clone();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn setup_ui(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        // ui camera
        .spawn(UiCameraBundle::default())
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::with_section(
                    "Button",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });
}

fn oscillation_system(
    time: Res<Time>,
    window: Res<Windows>,
    mut query: Query<&mut Camera, With<PerspectiveProjection>>,
) {
    for mut camera in query.iter_mut() {
        let aspect_ratio =
            window.get_primary().unwrap().width() / window.get_primary().unwrap().height();
        let oscillator =
            3.14 / 6.0 + (((time.seconds_since_startup() * 0.5).sin() + 1.0) * 3.14 / 4.0) as f32;
        camera.projection_matrix = Mat4::perspective_rh(oscillator, aspect_ratio, 1.0, 1000.0);
    }
}
