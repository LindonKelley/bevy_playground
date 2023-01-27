use std::hash::Hash;
use bevy::app::{App, CoreStage, Plugin};
use bevy::input::{Input, InputSystem};
use bevy::prelude::{IntoSystemDescriptor, KeyCode, MouseButton, Res, ResMut, Resource};
use bevy::utils::HashMap;
use derive_more::{From, TryInto};

#[derive(Clone, Default)]
pub struct KeyBindingPlugin<T: Send + Sync + Hash + Eq + Clone + Copy + 'static> {
    binds: KeyBindings<T>
}

impl <T: Send + Sync + Hash + Eq + Clone + Copy + 'static> KeyBindingPlugin<T> {
    /// Binds the provided `input` to the provided `bind`
    pub fn bind(mut self, input: impl Into<RawInput>, bind: T) -> Self {
        self.binds.bind(input, bind);
        self
    }

    /// Clears the binding to the provided `input`
    pub fn clear_bind(mut self, input: impl Into<RawInput>) -> Self {
        self.binds.clear_bind(input);
        self
    }

    /// Clears the binding to the provided `input` then binds `input` to the provided `bind`
    pub fn rebind(&mut self, input: impl Into<RawInput>, bind: T) -> &mut Self {
        self.binds.rebind(input, bind);
        self
    }
}

impl <T: Send + Sync + Hash + Eq + Clone + Copy + 'static> Plugin for KeyBindingPlugin<T> {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(self.binds.clone())
            .insert_resource(Input::<T>::default())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                map_keybinds::<T>.after(InputSystem)
            );
    }
}

pub fn map_keybinds<T: Send + Sync + Hash + Eq + Clone + Copy>(
    key_codes: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    key_bindings: Res<KeyBindings<T>>,
    mut binds: ResMut<Input<T>>
) {
    binds.clear();
    for (raw_input, bind) in &key_bindings.binds {
        match raw_input {
            RawInput::KeyCode(key_code) => {
                if key_codes.pressed(*key_code) {
                    binds.press(*bind);
                }
                if key_codes.just_released(*key_code) {
                    binds.release(*bind);
                }
            }
            RawInput::MouseButton(mouse_button) => {
                if mouse_buttons.pressed(*mouse_button) {
                    binds.press(*bind);
                }
                if mouse_buttons.just_released(*mouse_button) {
                    binds.release(*bind);
                }
            }
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct KeyBindings<T> {
    binds: HashMap<RawInput, T>
}

impl <T> KeyBindings<T> {
    /// Binds the provided `input` to the provided `bind`
    pub fn bind(&mut self, input: impl Into<RawInput>, bind: T) -> &mut Self {
        self.binds.insert(input.into(), bind);
        self
    }

    /// Clears the binding to the provided `input`
    pub fn clear_bind(&mut self, input: impl Into<RawInput>) -> &mut Self {
        self.binds.remove(&input.into());
        self
    }

    /// Clears the binding to the provided `input` then binds `input` to the provided `bind`
    pub fn rebind(&mut self, input: impl Into<RawInput>, bind: T) -> &mut Self {
        let raw_input = input.into();
        self.clear_bind(raw_input).bind(raw_input, bind)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, From, TryInto)]
pub enum RawInput {
    KeyCode(KeyCode),
    MouseButton(MouseButton)
}
