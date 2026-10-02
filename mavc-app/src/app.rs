//! Top-level MAVC app module; logic and frontend are maintained separately.

mod bridge;
mod formatting;
mod frontend;
mod i18n;
mod logic;
mod models;
mod playback;

pub use frontend::App;
