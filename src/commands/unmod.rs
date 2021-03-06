use super::{Channel, Encodable};
use std::io::{Result, Write};

/// Revoke moderator status from a user.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unmod<'a> {
    pub(crate) channel: &'a str,
    pub(crate) username: &'a str,
}

/// Revoke moderator status from a user.
///
/// Use [mods] to list the moderators of this channel.
///
/// [mods]: ./fn.mods.html
pub const fn unmod<'a>(channel: &'a str, username: &'a str) -> Unmod<'a> {
    Unmod { channel, username }
}

impl<'a> Encodable for Unmod<'a> {
    fn encode<W>(&self, buf: &mut W) -> Result<()>
    where
        W: Write + ?Sized,
    {
        write_cmd!(buf, Channel(self.channel) => "/unmod {}", self.username)
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unmod_encode() {
        test_encode(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }

    #[test]
    fn unmod_ensure_channel_encode() {
        test_encode(
            unmod("museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unmod_serde() {
        test_serde(
            unmod("#museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unmod_ensure_channel_serde() {
        test_serde(
            unmod("museun", "museun"),
            "PRIVMSG #museun :/unmod museun\r\n",
        );
    }
}
