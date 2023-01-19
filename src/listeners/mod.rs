use color_eyre::Result;
use serenity::{model::prelude::Message, prelude::Context};

mod wordle_reactions;

pub async fn dispatch(ctx: &Context, msg: Message) -> Result<()> {
    wordle_reactions::dispatch(ctx, msg).await?;
    Ok(())
}
