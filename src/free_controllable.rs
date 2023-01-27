use std::cmp::Ordering;
use std::f32::consts::{PI, TAU};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use bevy::app::{App, Plugin};
use bevy::input::Input;
use bevy::input::mouse::MouseMotion;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::{Component, EventReader, MouseButton, Query, Res, ResMut, Transform, With};
use bevy::utils::default;
use bevy::window::{CursorGrabMode, Windows};
use serde::{Deserialize, Serialize};
use crate::keybind::{KeyBindingPlugin, RawInput};

/// Adds free-moving controls to a 3D object, the generic `T` specifies what component this plugin
/// will target with it's control. This plugin can be initialized in two ways:
///
/// * No default bindings [FreeControlPlugin::new]
/// * regular WASD controls, left shift for down, space for up [FreeControlPlugin::default]
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
            .bind(Space, FreeControls::Up)
            .bind(MouseButton::Left, FreeControls::Locked)
            .bind(Escape, FreeControls::Unlock);

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
    Locked,
    Unlock,
    #[allow(non_camel_case_types)]
    __phantom(PhantomData<fn(T)>)
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
            FreeControls::Locked => 6,
            FreeControls::Unlock => 7,
            FreeControls::__phantom(_) => 8,
        }
    }
}

pub fn free_controls<T: Component>(mut windows: ResMut<Windows>, mut ev_motion: EventReader<MouseMotion>, binds: Res<Input<FreeControls<T>>>, mut query: Query<&mut Transform, With<T>>) {
    // todo remove forced usage of MouseMotion, likely requires some rewriting of KeyBindingPlugin
    // todo camera speed and sensitivity settings
    // todo needs to handle multiple windows
    let window = windows.get_primary_mut().unwrap();
    // todo lock and unlock should probably be state based, and sent to the camera via Events
    // todo need to add aggressive locking, but leave it as an option in game
    // matches! seems to be necessary here, as locking the cursor grab mode more than once causes
    // it to behave as if it's unlocked, at least on my system, Arch Linux, (KDE x11)
    if binds.just_pressed(FreeControls::Locked) && !matches!(window.cursor_grab_mode(), CursorGrabMode::Locked) {
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
        window.set_cursor_position(Vec2::new(window.width() / 2.0, window.height() / 2.0));
        window.set_cursor_visibility(false);
    }
    if binds.just_pressed(FreeControls::Unlock) {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);
    }

    if matches!(window.cursor_grab_mode(), CursorGrabMode::Locked) {
        let mut rotation_move = Vec2::ZERO;
        for motion in ev_motion.iter() {
            rotation_move += motion.delta;
        }
        rotation_move *= 0.5;

        let mut transform = query.single_mut();
        if rotation_move.length_squared() > 0.0 {
            let yaw = Quat::from_rotation_y(-rotation_move.x / window.width() * TAU);
            let pitch = Quat::from_rotation_x(-rotation_move.y / window.height() * PI);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        }

        let mut handle = |input, f: fn(&Transform) -> Vec3| {
            if binds.pressed(input) {
                let delta = f(&transform) * 0.5;
                transform.translation += delta;
            }
        };

        {
            use FreeControls::*;

            handle(Forward, Transform::forward);
            handle(Backward, Transform::back);
            handle(Left, Transform::left);
            handle(Right, Transform::right);
            handle(Up, Transform::up);
            handle(Down, Transform::down);
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
        matches!(self, other)
    }
}

impl <T> Eq for FreeControls<T> {}

impl <T> Hash for FreeControls<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_num().hash(state)
    }
}
