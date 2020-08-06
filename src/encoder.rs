use futures_lite::{AsyncWrite, AsyncWriteExt};
use std::{
    io::{Result as IoResult, Write},
    pin::Pin,
    task::{Context, Poll},
};

/// A trait to allow writing messags to any `std::io::Write` implementation
pub trait Encodable {
    /// Encode this message to the provided `std::io::Write` implementation
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> IoResult<()>;
}

impl<T> Encodable for &T
where
    T: Encodable + ?Sized,
{
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> IoResult<()> {
        <_ as Encodable>::encode(*self, buf)
    }
}

impl Encodable for str {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> IoResult<()> {
        buf.write_all(self.as_bytes())
    }
}

impl Encodable for String {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> IoResult<()> {
        buf.write_all(self.as_bytes())
    }
}

impl Encodable for [u8] {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> IoResult<()> {
        buf.write_all(self)
    }
}

impl Encodable for Vec<u8> {
    fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> IoResult<()> {
        buf.write_all(self)
    }
}

/// A synchronous encoder
pub struct Encoder<W> {
    writer: W,
}

impl<W: Write> Encoder<W> {
    /// Create a new Encoder over this `std::io::Write` instance
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Get the inner `std::io::Write` instance out
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Encode this `Encodable` message to the writer and flushes it.
    pub fn encode<M>(&mut self, msg: M) -> IoResult<()>
    where
        M: Encodable,
    {
        msg.encode(&mut self.writer)?;
        self.writer.flush()
    }
}

impl<W: Write> Write for Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.writer.flush()
    }
}

pin_project_lite::pin_project! {
    /// An asynchronous encoder.
    pub struct AsyncEncoder<W> {
        #[pin]
        writer: W,
        pos: usize,
        data: Vec<u8>
    }
}

impl<W: AsyncWrite + Unpin> AsyncEncoder<W> {
    /// Create a new Encoder over this `futures::io::AsyncWrite` instance
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            pos: 0,
            data: Vec::with_capacity(1024),
        }
    }

    /// Get the inner `futures::io::AsyncWrite` instance out
    ///
    /// This writes and flushes any buffered data before it consumes self.
    pub async fn into_inner(mut self) -> IoResult<W> {
        if self.data.is_empty() {
            return Ok(self.writer);
        }

        let data = std::mem::take(&mut self.data);
        self.writer.write_all(&data).await?;
        self.writer.flush().await?;
        Ok(self.writer)
    }

    /// Encode this `Encodable` message to the writer.
    ///
    /// This flushes the data before returning
    pub async fn encode<M>(&mut self, msg: M) -> IoResult<()>
    where
        M: Encodable,
    {
        msg.encode(&mut self.data)?;
        self.writer.write_all(&self.data[self.pos..]).await?;
        self.writer.flush().await?;
        self.pos = {
            self.data.clear();
            self.data.len()
        };
        Ok(())
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncEncoder<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        let this = self.project();
        this.writer.poll_write(ctx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();
        this.writer.poll_flush(ctx)
    }

    fn poll_close(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<IoResult<()>> {
        let this = self.project();
        this.writer.poll_close(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct JoinCommand<'a> {
        channel: &'a str,
    }

    fn join(channel: &str) -> JoinCommand<'_> {
        JoinCommand { channel }
    }

    impl<'a> Encodable for JoinCommand<'a> {
        fn encode<W: Write + ?Sized>(&self, buf: &mut W) -> IoResult<()> {
            write!(buf, "JOIN {}\r\n", &self.channel)
        }
    }

    #[test]
    fn encoder() {
        let mut encoder = Encoder::new(vec![]);

        encoder.encode(join("#museun")).unwrap();
        encoder.encode(join("#shaken_bot")).unwrap();

        encoder.flush().unwrap();

        // using into_inner here instead of &mut borrowing the vec and dropping the encoder
        let out = encoder.into_inner();
        let s = std::str::from_utf8(&out).unwrap();
        assert_eq!(s, "JOIN #museun\r\nJOIN #shaken_bot\r\n");
    }

    #[test]
    fn encoder_async() {
        let fut = async move {
            let mut output = vec![];
            {
                let mut encoder = AsyncEncoder::new(&mut output);

                encoder.encode(join("#museun")).await.unwrap();
                encoder.encode(join("#shaken_bot")).await.unwrap();

                encoder.flush().await.unwrap();
            }

            let s = std::str::from_utf8(&output).unwrap();
            assert_eq!(s, "JOIN #museun\r\nJOIN #shaken_bot\r\n");
        };
        async_executor::LocalExecutor::new().run(fut);
    }

    #[test]
    fn encodable_builtin() {
        fn check<T: Encodable + AsRef<[u8]> + ?Sized>(input: &T) {
            let mut output = vec![];
            let mut encoder = Encoder::new(&mut output);
            encoder.encode(input).unwrap();
            assert_eq!(output, input.as_ref());
        }

        let input = "hello world\r\n";
        check(&input);
        check(&input.to_string());
        check(&input.as_bytes());
        check(&input.as_bytes().to_vec());
    }
}
