use serde::{Deserialize, Serialize};
use strum::Display;

/// Actions are user-initiated events available for consumption by any in-focus component.
/// Actions are configured by keybinds.
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    MakeSelection,
    ViewStructure,
    ViewIndexes,
    NavDown,
    NavUp,
    NavLeft,
    NavRight,
    PageLeft,
    PageRight,
    Yank,
    Search,
    Clear,
}
