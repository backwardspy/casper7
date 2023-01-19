use color_eyre::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    model::prelude::{Message, ReactionType},
    prelude::Context,
};

struct Reaction {
    pattern: Regex,
    emoji: String,
}

fn react(pattern: &str, emoji: &str) -> Reaction {
    Reaction {
        #[allow(clippy::expect_used)] // skill issue
        pattern: Regex::new(pattern).expect("failed to compile regex: {pattern}"),
        emoji: emoji.to_owned(),
    }
}

lazy_static! {
    static ref REACTIONS: Vec<Reaction> = vec![
        react(r"wordle \d+ [1-6]/6", "🧠"),
        react(r"wordle \d+ 1/6", "1️⃣"),
        react(r"wordle \d+ 2/6", "2️⃣"),
        react(r"wordle \d+ X/6", "🐌"),
        react(r"daily duotrigordle #\d+\nguesses: \d+/37", "🧠"),
        react(r"daily duotrigordle #\d+\nguesses: X/37", "🐌"),
        react(r"scholardle \d+ [1-6]/6", "🎓"),
        react(r"scholardle \d+ 1/6", "1️⃣"),
        react(r"scholardle \d+ 2/6", "2️⃣"),
        react(r"scholardle \d+ X/6", "🐌"),
        react(r"worldle #\d+ [1-6]/6 \(100%\)", "🗺️"),
        react(r"worldle #\d+ X/6 \(\d+%\)", "🐌"),
        react(r"waffle\d+ [0-5]/5", "🧇"),
        react(r"waffle\d+ 5/5", "⭐"),
        react(r"waffle\d+ X/5", "🐌"),
        react(r"#wafflesilverteam", "🥈"),
        react(r"#wafflegoldteam", "🥇"),
        react(r"#wafflecenturion", "🌟"),
        react(r"#wafflemaster", "🏆"),
        react(r"flowdle \d+ \[\d+ moves\]", "🚰"),
        react(r"flowdle \d+ \[failed\]", "🐌"),
        react(r"jurassic wordle \(game #\d+\) - [1-8] / 8", "🦕"),
        react(r"jurassic wordle \(game #\d+\) - X / 8", "🐌"),
        react(r"jungdle \(game #\d+\) - [1-8] / 8", "🦁"),
        react(r"jungdle \(game #\d+\) - X / 8", "🐌"),
        react(r"dogsdle \(game #\d+\) - [1-8] / 8", "🐶"),
        react(r"dogsdle \(game #\d+\) - X / 8", "🐌"),
        react(r"framed #\d+.*\n+.*🎥 [🟥⬛ ]*🟩", "🎬"),
        react(r"framed #\d+.*\n+.*🎥 [🟥⬛ ]+$", "🐌"),
        react(r"moviedle #[\d-]+.*\n+.*🎥[🟥⬜⬛️ ]*🟩", "🎬"),
        react(r"moviedle #[\d-]+.*\n+.*🎥[🟥⬜⬛️ ]+$", "🐌"),
        react(r"posterdle #[\d-]+.*\n+ ⌛ .*\n 🍿.+🟩", "📯"),
        react(r"posterdle #[\d-]+.*\n+ ⌛ 0️⃣ .*\n 🍿.+🟩", "0️⃣"),
        react(r"posterdle #[\d-]+.*\n+ ⌛ .*\n 🍿 [⬜️🟥⬛️ ]+$", "🐌"),
        react(r"namethatride #[\d-]+.*\n+ ⌛ .*\n 🚗.+🟩", "🚙"),
        react(r"namethatride #[\d-]+.*\n+ ⌛ .*\n 🚗 [⬜️🟥⬛️ ]+$", "🐌"),
        react(r"heardle #\d+.*\n+.*🟩", "👂"),
        react(r"heardle #\d+.*\n+🔇", "🐌"),
        react(r"flaggle .*\n+.*\d+ pts", "⛳"),
        react(r"flaggle .*\n+.*gave up", "🐌"),
        react(r"#Polygonle \d+ [1-6]/6[^🟧]+?🟩", "🔷"),
        react(r"#Polygonle \d+ [1-6]/6[^🟩]+?🟧", "🔶"),
        react(r"#Polygonle \d+ X/6", "🐌"),
        react(r"#GuessTheGame #\d+.*\n+.*🎮[🟥⬛ ]*🟩", "🎮"),
        react(r"#GuessTheGame #\d+.*\n+.*🎮 [🟥⬛ ]+$", "🐌"),
        react(r"https://squaredle\.app/ \d+/\d+:", "🟩"),
        react(r"https://squaredle\.app/ .*[^📖]*📖", "📖"),
        react(r"https://squaredle\.app/ .*[^⏱️]*⏱️", "⏱️"),
        react(r"https://squaredle\.app/ .*[^🎯]*🎯", "🎯"),
        react(r"https://squaredle\.app/ .*[^🔥]*🔥", "🔥"),
        react(r"Episode #\d+\n+📺 .*🟩", "📺"),
        react(r"Episode #\d+\n+📺 [^🟩]+$", "🐌"),
    ];
}

pub async fn dispatch(ctx: &Context, msg: Message) -> Result<()> {
    for reaction in REACTIONS.iter() {
        if reaction.pattern.is_match(&msg.content) {
            msg.react(ctx.clone(), ReactionType::Unicode(reaction.emoji.clone()))
                .await?;
            break;
        }
    }
    Ok(())
}
