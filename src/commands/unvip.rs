use crate::Encodable;
use std::{
    borrow::Cow,
    io::{Result, Write},
};

use super::ByteWriter;

/// Revoke VIP status from a user.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
pub struct Unvip<'a> {
    pub(crate) channel: Cow<'a, str>,
    pub(crate) username: &'a str,
}

/// Revoke VIP status from a user.
///
/// Use [vips] to list the VIPs of this channel.
///
/// [vips]: ./fn.vips.html
pub fn unvip<'a>(channel: &'a str, username: &'a str) -> Unvip<'a> {
    let channel = super::make_channel(channel);
    Unvip { channel, username }
}

impl<'a> Encodable for Unvip<'a> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> Result<()> {
        ByteWriter::new(buf).command(&&*self.channel, &[&"/unvip", &self.username])
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn unvip_encode() {
        test_encode(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }

    #[test]
    fn unvip_ensure_channel_encode() {
        test_encode(
            unvip("museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unvip_serde() {
        test_serde(
            unvip("#museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn unvip_ensure_channel_serde() {
        test_serde(
            unvip("museun", "museun"),
            "PRIVMSG #museun :/unvip museun\r\n",
        );
    }
}
