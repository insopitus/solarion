//! Level curve and stage rewards.
//!
//! Characters gain experience when a stage is cleared. The stage also drops
//! experience potions, which are banked for the (future) character screen.

use bevy::prelude::*;

pub const POTIONS_PER_STAGE: u32 = 3;

#[derive(Resource, Debug, Default)]
pub struct Progression {
    pub potions: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelProgress {
    pub level: u32,
    pub xp: u32,
}

impl Default for LevelProgress {
    fn default() -> Self {
        Self { level: 1, xp: 0 }
    }
}

pub fn xp_to_next_level(level: u32) -> u32 {
    100 + 60 * level.saturating_sub(1)
}

/// Adds experience, potentially gaining multiple levels.
/// Returns the number of levels gained.
pub fn add_experience(progress: &mut LevelProgress, amount: u32) -> u32 {
    let mut gained = 0;
    progress.xp = progress.xp.saturating_add(amount);

    loop {
        let needed = xp_to_next_level(progress.level);
        if progress.xp < needed {
            break;
        }
        progress.xp -= needed;
        progress.level += 1;
        gained += 1;
        // Safety valve for absurd amounts of experience.
        if gained > 1000 {
            break;
        }
    }

    gained
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_up_once_when_threshold_is_crossed() {
        let mut progress = LevelProgress::default();
        let gained = add_experience(&mut progress, 100);
        assert_eq!(progress.level, 2);
        assert_eq!(progress.xp, 0);
        assert_eq!(gained, 1);
    }

    #[test]
    fn keeps_leftover_experience() {
        let mut progress = LevelProgress::default();
        add_experience(&mut progress, 130);
        assert_eq!(progress.level, 2);
        assert_eq!(progress.xp, 30);
    }

    #[test]
    fn can_gain_multiple_levels() {
        let mut progress = LevelProgress::default();
        let gained = add_experience(&mut progress, 1000);
        assert!(gained >= 2);
        assert!(progress.level >= 3);
    }
}
