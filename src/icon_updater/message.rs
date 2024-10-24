use serenity::all::{Attachment, Message};

pub(crate) trait MessageExt {
    fn images(&self) -> impl Iterator<Item = &Attachment>;
}

impl MessageExt for Message {
    fn images(&self) -> impl Iterator<Item = &Attachment> {
        self.attachments
            .iter()
            .filter(|att| att.width.is_some() && att.height.is_some())
    }
}
