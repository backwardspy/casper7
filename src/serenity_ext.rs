use serenity::model::{
    prelude::{
        interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        PartialChannel, Role,
    },
    user::User,
};

pub(crate) fn get_integer_option(options: &[CommandDataOption], index: usize) -> Option<i64> {
    let option = match options.get(index) {
        Some(option) => option,
        None => return None,
    };

    let option = match &option.resolved {
        Some(option) => option,
        None => return None,
    };

    if let CommandDataOptionValue::Integer(value) = option {
        Some(*value)
    } else {
        None
    }
}

pub(crate) fn get_user_option(options: &[CommandDataOption], index: usize) -> Option<&User> {
    let option = match options.get(index) {
        Some(option) => option,
        None => return None,
    };

    let option = match &option.resolved {
        Some(option) => option,
        None => return None,
    };

    if let CommandDataOptionValue::User(user, _member) = option {
        Some(user)
    } else {
        None
    }
}

pub(crate) fn get_channel_option(
    options: &[CommandDataOption],
    index: usize,
) -> Option<&PartialChannel> {
    let option = match options.get(index) {
        Some(option) => option,
        None => return None,
    };

    let option = match &option.resolved {
        Some(option) => option,
        None => return None,
    };

    if let CommandDataOptionValue::Channel(channel) = option {
        Some(channel)
    } else {
        None
    }
}

pub(crate) fn get_role_option(options: &[CommandDataOption], index: usize) -> Option<&Role> {
    let option = match options.get(index) {
        Some(option) => option,
        None => return None,
    };

    let option = match &option.resolved {
        Some(option) => option,
        None => return None,
    };

    if let CommandDataOptionValue::Role(role) = option {
        Some(role)
    } else {
        None
    }
}
