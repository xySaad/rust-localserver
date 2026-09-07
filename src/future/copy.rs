use std::io;

use crate::future::{AsyncRead, AsyncWrite};

pub async fn copy<R: AsyncRead, W: AsyncWrite>(reader: &mut R, writer: &mut W) -> io::Result<u64> {
    let mut buf = [0u8; 8192];
    let mut total = 0u64;

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        let mut written = 0;
        while written < n {
            let m = writer.write(&buf[written..n]).await?;
            if m == 0 {
                return Err(io::Error::new(io::ErrorKind::WriteZero, "write returned 0"));
            }
            written += m;
        }

        total += n as u64;
    }

    Ok(total)
}
