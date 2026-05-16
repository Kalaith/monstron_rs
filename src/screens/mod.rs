pub mod hatchery;
pub mod menu;
pub mod placeholder;
pub mod stable;
pub mod tower;
pub mod town;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppScreen {
    MainMenu,
    Town,
    Hatchery,
    Stable,
    DungeonPrep,
    Tower,
    Combat,
    EndOfDay,
}
