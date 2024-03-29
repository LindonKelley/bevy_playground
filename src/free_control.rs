use std::cmp::Ordering;
use std::f32::consts::{PI, TAU};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use bevy::app::{App, Plugin};
use bevy::input::Input;
use bevy::input::mouse::MouseMotion;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{Component, EventReader, Query, Res, ResMut, Resource, Transform, With};
use bevy::utils::default;
use bevy::window::{CursorGrabMode, Windows};
use serde::{Deserialize, Serialize};
use crate::keybind::{KeyBindingPlugin, RawInput};

/// Adds free-moving controls to 3D objects, specifically all entities with the component
/// [Transform] and the provided generic [T]. This plugin can be initialized in two ways:
///
/// * No default bindings [FreeControlPlugin::new]
/// * regular WASD controls, left shift for down, space for up [FreeControlPlugin::default]
///
/// The [FreeControlConfig] resource can be used to control the speed and sensitivity of the
/// entities
pub struct FreeControlPlugin<T: Component> {
    key_bindings: KeyBindingPlugin<FreeControls<T>>,
    __phantom: PhantomData<fn(T)>
}

impl <T: Component> FreeControlPlugin<T> {
    /// Creates a new `FreeControlPlugin`, without any default bindings
    pub fn new() -> Self {
        Self {
            key_bindings: KeyBindingPlugin::default(),
            __phantom: default()
        }
    }

    pub fn bind(mut self, input: impl Into<RawInput>, bind: FreeControls<T>) -> Self {
        self.key_bindings = self.key_bindings.bind(input, bind);
        self
    }
}

impl <T: Component> Clone for FreeControlPlugin<T> {
    fn clone(&self) -> Self {
        Self {
            key_bindings: self.key_bindings.clone(),
            __phantom: self.__phantom
        }
    }
}

impl <T: Component> Default for FreeControlPlugin<T> {
    fn default() -> Self {
        use bevy::prelude::KeyCode::*;

        let key_bindings = KeyBindingPlugin::default()
            .bind(W, FreeControls::Forward)
            .bind(S, FreeControls::Backward)
            .bind(A, FreeControls::Left)
            .bind(D, FreeControls::Right)
            .bind(LShift, FreeControls::Down)
            .bind(Space, FreeControls::Up);

        Self {
            key_bindings,
            __phantom: default()
        }
    }
}

impl <T: Component> Plugin for FreeControlPlugin<T> {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(self.key_bindings.clone())
            .add_system(free_controls::<T>);
        if !app.world.contains_resource::<FreeControlConfig<T>>() {
            app.insert_resource(FreeControlConfig::<T>::default());
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub enum FreeControls<T> {
    #[default]
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
    #[allow(non_camel_case_types)]
    __phantom(PhantomData<fn(T)>)
}

#[derive(Resource)]
pub struct FreeControlConfig<T> {
    pub forward_speed: f32,
    pub backward_speed: f32,
    pub left_speed: f32,
    pub right_speed: f32,
    pub up_speed: f32,
    pub down_speed: f32,

    pub left_sensitivity: f32,
    pub right_sensitivity: f32,
    pub up_sensitivity: f32,
    pub down_sensitivity: f32,

    pub __phantom: PhantomData<fn(T)>
}

impl <T> Default for FreeControlConfig<T> {
    fn default() -> Self {
        Self {
            forward_speed: 0.5,
            backward_speed: 0.5,
            left_speed: 0.5,
            right_speed: 0.5,
            up_speed: 0.5,
            down_speed: 0.5,

            left_sensitivity: 0.5 * TAU,
            right_sensitivity: 0.5 * TAU,
            up_sensitivity: 0.5 * PI,
            down_sensitivity: 0.5 * PI,

            __phantom: default()
        }
    }
}

pub fn free_controls<T: Component>(
    mut windows: ResMut<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    config: Res<FreeControlConfig<T>>,
    binds: Res<Input<FreeControls<T>>>,
    mut free_control: Query<&mut Transform, With<T>>
) {
    // todo remove forced usage of MouseMotion, likely requires some rewriting of KeyBindingPlugin
    // todo needs to handle multiple windows, going to wait until Bevy updates to having Windows as Entities
    let window = windows.get_primary_mut().unwrap();

    if matches!(window.cursor_grab_mode(), CursorGrabMode::Locked) {
        let mut rotation_move = Vec2::ZERO;
        for motion in ev_motion.iter() {
            let Vec2 {x, y} = motion.delta;
            if x < 0.0 {
                rotation_move.x += x * config.left_sensitivity;
            } else {
                rotation_move.x += x * config.right_sensitivity;
            }
            if y < 0.0 {
                rotation_move.y += y * config.up_sensitivity;
            } else {
                rotation_move.y += y * config.down_sensitivity;
            }
        }

        for mut transform in &mut free_control {
            let yaw = Quat::from_rotation_y(-rotation_move.x / window.width());
            let pitch = Quat::from_rotation_x(-rotation_move.y / window.height());
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis

            let mut handle = |input, f: fn(&Transform) -> Vec3, speed| {
                if binds.pressed(input) {
                    let delta = f(&transform) * speed;
                    transform.translation += delta;
                }
            };

            {
                use FreeControls::*;

                handle(Forward, Transform::forward, config.forward_speed);
                handle(Backward, Transform::back, config.backward_speed);
                handle(Left, Transform::left, config.left_speed);
                handle(Right, Transform::right, config.right_speed);
                handle(Up, Transform::up, config.up_speed);
                handle(Down, Transform::down, config.down_speed);
            }
        }
    }
}

impl <T> FreeControls<T> {
    fn to_num(self) -> u32 {
        match self {
            FreeControls::Forward => 0,
            FreeControls::Backward => 1,
            FreeControls::Left => 2,
            FreeControls::Right => 3,
            FreeControls::Up => 4,
            FreeControls::Down => 5,
            FreeControls::__phantom(_) => 6,
        }
    }
}

// I'm forced to manually implement all of these since Rust's derive for them forces T to also have
// the trait, which is not necessary when there's a PhantomData involved

impl <T> Copy for FreeControls<T> {}

impl<T> Clone for FreeControls<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <T> PartialOrd for FreeControls<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <T> Ord for FreeControls<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.to_num();
        let b = other.to_num();
        a.cmp(&b)
    }
}

impl <T> PartialEq for FreeControls<T> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.to_num();
        let b = other.to_num();
        a == b
    }
}

impl <T> Eq for FreeControls<T> {}

impl <T> Hash for FreeControls<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_num().hash(state)
    }
}
