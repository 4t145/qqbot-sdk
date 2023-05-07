#[repr(transparent)]
pub struct Intends(pub u32);

macro_rules! def_intends {
    ($($ident:ident, $offset:expr)*) => {
        impl Intends {
            $(
                pub const $ident: u32 = (1 << $offset);
            )*
        }
    };

}

def_intends! {
    GUILDS, 0
    GUILD_MEMBERS, 1
    GUILD_MESSAGES, 9
    GUILD_MESSAGE_REACTIONS, 10
    DIRECT_MESSAGE, 12
    INTERACTION, 26
    MESSAGE_AUDIT, 27
    FORUMS_EVENT, 28
    AUDIO_ACTION, 29
    PUBLIC_GUILD_MESSAGES, 30
}
