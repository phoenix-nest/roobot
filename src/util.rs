use serenity::all::{ChannelId, Http, Message, MessageId};

pub(crate) fn find_last_existing_msg(
    http: &Http,
    channel: ChannelId,
    msgs: impl IntoIterator<Item = MessageId>,
) -> Option<MessageId> {
    todo!()
}
