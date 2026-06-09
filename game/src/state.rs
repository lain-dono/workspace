use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<AppState>().add_computed_state::<InGame>();

    #[cfg(feature = "dev_mode")]
    app.add_systems(Update, bevy::dev_tools::states::log_transitions::<AppState>);
}

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Debug, Hash)]
#[states(scoped_entities)]
pub enum AppState {
    /// Shows splash screen
    ///
    /// Now skips [`GameState::Loading`] and run [`GameState::MainMenu`]
    #[default]
    Splash,

    /// Shows loading process
    Loading,

    /// Starting game, goto settings, quit from game
    MainMenu,

    /// State during gameplay
    InGame { paused: bool },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InGame;

impl ComputedStates for InGame {
    type SourceStates = AppState;

    fn compute(sources: Self::SourceStates) -> Option<InGame> {
        match sources {
            Self::SourceStates::InGame { .. } => Some(InGame),
            _ => None,
        }
    }
}

// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
// pub enum IsPaused {
//     Running,
//     Paused,
// }

// impl ComputedStates for IsPaused {
//     type SourceStates = AppState;

//     fn compute(sources: Self::SourceStates) -> Option<Self> {
//         match sources {
//             Self::SourceStates::InGame { paused: true, .. } => Some(Self::Paused),
//             Self::SourceStates::InGame { paused: false, .. } => Some(Self::Running),
//             _ => None,
//         }
//     }
// }
