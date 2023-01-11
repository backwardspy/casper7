use serenity::{model::prelude::Message, prelude::Context};

mod wordle_reactions;

pub(crate) fn dispatch(ctx: Context, msg: Message) {
    wordle_reactions::dispatch(ctx, msg);
}
