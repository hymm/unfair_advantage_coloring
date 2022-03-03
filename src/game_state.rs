#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Loading,
    StartMenu,
    Painting,
    Results,
    LeaderBoard,
}
