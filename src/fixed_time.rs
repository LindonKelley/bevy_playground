use std::time::Duration;
use bevy::app::{App, Plugin};
use bevy::prelude::{Res, ResMut};
use bevy::time::{Time, TimeUpdateStrategy};

/// Ensures that each tick of Bevy's Time is 1/60 seconds after the last, irregardless of
/// actual time passed (which should be roughly the same).
///
/// The main reason for doing this is to keep Rapier physics deterministic, and to keep anything
/// else in the world from looking wonky next to anything controlled by those physics
/// (as opposed telling Rapier to use TimestepMode::Interpolated since getting time from Rapier
/// isn't very straightforward like it is with Bevy)
pub struct FixedTimePlugin;

impl Plugin for FixedTimePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fixed_time_step);
    }
}

fn fixed_time_step(time: Res<Time>, mut time_update_strategy: ResMut<TimeUpdateStrategy>) {
    *time_update_strategy = TimeUpdateStrategy::ManualInstant(time.last_update().unwrap() + Duration::from_secs_f64(1.0 / 60.0));
}
