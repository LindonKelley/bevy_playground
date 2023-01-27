mod keybind;
mod free_control;
mod fixed_time;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::pbr::{DirectionalLight, DirectionalLightBundle, PbrBundle, StandardMaterial};
use bevy::prelude::{Camera3dBundle, Color, Commands, Component, Mesh, ResMut, shape, Transform};
use bevy::utils::default;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use crate::fixed_time::FixedTimePlugin;
use crate::free_control::FreeControlPlugin;

fn main() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(FixedTimePlugin)
        .add_plugin(FreeControlPlugin::<FreeCam>::default())
        .add_startup_system(setup_camera_and_light)
        .add_startup_system(setup_environment);
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

fn setup_environment(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let meshes = &mut meshes;
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Box::new(2.0, 2.0, 2.0).into()),
            material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
            ..default()
        });
}
