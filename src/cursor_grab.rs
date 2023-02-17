use bevy::app::{App, Plugin};
use bevy::ecs::schedule::ShouldRun;
use bevy::log::info;
use bevy::math::Vec2;
use bevy::prelude::{EventReader, Local, Res, ResMut, Resource, State, SystemSet, Window};
use bevy::window::{CursorGrabMode, WindowFocused, Windows};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Resource)]
pub enum CursorGrab {
    Active,
    Inactive
}

impl CursorGrab {
    pub fn is_active(&self) -> bool {
        *self == CursorGrab::Active
    }

    pub fn is_inactive(&self) -> bool {
        *self == CursorGrab::Inactive
    }

    pub fn activate(&mut self) {
        *self = CursorGrab::Active;
    }

    pub fn deactivate(&mut self) {
        *self = CursorGrab::Inactive;
    }
}

/// A plugin to handle cursor grabbing, or locking the cursor to the center of the window.
/// This functionality is initialized as inactive, use the Resource `CursorGrab` to switch between
/// active and inactive.
///
/// See [cursor_grab] for details on how this works.
pub struct CursorGrabPlugin;

impl Plugin for CursorGrabPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CursorGrab::Inactive)
            .add_system(cursor_grab);
    }
}

/// The sole system for [CursorGrabPlugin].
///
/// A couple things to note about how the grabbing works (assuming that CursorGrab is Active):
///  if focus on the window is just lost, the cursor is released.
///  if focus on the window is just gained, the cursor is grabbed.
/// These above two allow alt-tabbing to work properly, otherwise some platforms will just keep
///  bringing the application back into view (preventing alt tabbing)
pub fn cursor_grab(
    cursor_grab: Res<CursorGrab>,
    mut focus_events: EventReader<WindowFocused>,
    mut windows: ResMut<Windows>
) {
    // todo needs to handle multiple windows, going to wait until Bevy updates to having Windows as Entities
    let release = |window: &mut Window| {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        // some platforms will not actually lock the cursor, and in those cases this will at least
        // provide the illusion that the mouse stays centered.
        // as an example: user using a 3d camera, opens a menu, the mouse will always show up in the
        // center of the screen, as opposed to any other random location that the 3d camera would've
        // ended up leaving the cursor.
        window.set_cursor_position(Vec2::new(window.width() / 2.0, window.height() / 2.0));
        window.set_cursor_visibility(true);
    };
    let grab = |window: &mut Window| {
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
        window.set_cursor_position(Vec2::new(window.width() / 2.0, window.height() / 2.0));
        window.set_cursor_visibility(false);
    };

    let window = windows.primary_mut();

    match *cursor_grab {
        CursorGrab::Active => {
            if cursor_grab.is_changed() && window.is_focused() {
                grab(window);
                return;
            }
        }
        CursorGrab::Inactive => {
            if cursor_grab.is_changed() {
                if window.cursor_grab_mode() != CursorGrabMode::None {
                    release(window);
                }
            }
            return;
        }
    }

    let focus_changed = focus_events
        .iter()
        .filter(|event| event.id.is_primary())
        .next()
        .is_some();

    if focus_changed {
        if window.is_focused() {
            grab(window);
        } else {
            release(window);
        }
    }
}
