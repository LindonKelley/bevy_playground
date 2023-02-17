mod keybind;
mod free_control;
mod fixed_time;
mod cursor_grab;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::pbr::{DirectionalLight, DirectionalLightBundle, PbrBundle, StandardMaterial};
use bevy::prelude::{Camera3dBundle, Color, Commands, Component, IntoSystemDescriptor, KeyCode, Mesh, MouseButton, Res, ResMut, shape, Transform};
use bevy::utils::default;
use bevy::window::{WindowMode, Windows};
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use crate::cursor_grab::{cursor_grab, CursorGrab, CursorGrabPlugin};
use crate::fixed_time::FixedTimePlugin;
use crate::free_control::FreeControlPlugin;

fn main() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(FixedTimePlugin)
        .add_plugin(FreeControlPlugin::<FreeCam>::default())
        .add_plugin(CursorGrabPlugin)
        .add_startup_system(setup_camera_and_light)
        .add_startup_system(setup_environment)
        .add_system(toggle_cursor_grab.before(cursor_grab))
        .add_system(toggle_fullscreen);
    app.run();
}

#[derive(Component)]
pub struct FreeCam;

fn setup_camera_and_light(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 20.0).looking_at(Vec3::Y * 5.0, Vec3::Y),
        ..default()
    })
        .insert(FreeCam);

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_environment(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut cursor_grab: ResMut<CursorGrab>) {
    let meshes = &mut meshes;
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Box::new(2.0, 2.0, 2.0).into()),
            material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
            ..default()
        });
    cursor_grab.activate();
}

fn toggle_cursor_grab(mut cursor_grab: ResMut<CursorGrab>, key_codes: Res<Input<KeyCode>>, mouse_buttons: Res<Input<MouseButton>>) {
    if mouse_buttons.just_pressed(MouseButton::Left) && cursor_grab.is_inactive() {
        info!("activated");
        cursor_grab.activate();
    }
    if key_codes.just_pressed(KeyCode::Escape) && cursor_grab.is_active() {
        info!("deactivated");
        cursor_grab.deactivate();
    }
}

fn toggle_fullscreen(key_codes: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if key_codes.just_pressed(KeyCode::F11) {
        if !matches!(windows.primary().mode(), WindowMode::BorderlessFullscreen) {
            windows.primary_mut().set_mode(WindowMode::BorderlessFullscreen);
        } else {
            windows.primary_mut().set_mode(WindowMode::Windowed);
        }
    }
}
