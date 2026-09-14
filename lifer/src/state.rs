use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<AppState>()
        .add_computed_state::<InGame>()
        .enable_state_scoped_entities::<InGame>();
}

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Debug, Hash)]
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
    InGame,
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
// enum IsPaused {
//     NotPaused,
//     Paused,
// }

// impl ComputedStates for IsPaused {
//     type SourceStates = GameState;

//     fn compute(sources: Self::SourceStates) -> Option<Self> {
//         match sources {
//             Self::SourceStates::InGame { paused: true, .. } => Some(Self::Paused),
//             Self::SourceStates::InGame { paused: false, .. } => Some(Self::NotPaused),
//             _ => None,
//         }
//     }
// }
