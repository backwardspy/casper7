pub mod meatball;
pub mod wordle;

pub fn commands() -> Vec<poise::Command<crate::Bot, color_eyre::eyre::ErrReport>> {
    vec![meatball::commands::meatball()]
}
