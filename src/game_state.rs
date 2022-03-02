#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Loading,
    StartMenu,
    NameEntry,
    ShowBrush,
    Painting,
    Results,
    LeaderBoard,
}
