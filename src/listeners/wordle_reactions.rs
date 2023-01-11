use serenity::{model::prelude::Message, prelude::Context};

pub(crate) fn dispatch(ctx: Context, msg: Message) {
    dbg!(msg.content);
}
