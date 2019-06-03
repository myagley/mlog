use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::fmt;
use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use failure::ResultExt;

mod error;

use self::error::{Error, ErrorKind};

const MAGIC: u32 = 0x4d4c4f47;
const VERSION: u8 = 0x1;

struct Log {}

/// File header for log
#[derive(Debug, PartialEq)]
struct Header {
    len: u64,
    start: LogIndex,
    end: LogIndex,
}

impl Header {
    fn from_read<R: Read>(reader: &mut R) -> Result<Header, Error> {
        let magic = reader
            .read_u32::<BigEndian>()
            .context(ErrorKind::ReadHeader)?;
        if magic != MAGIC {
            Err(ErrorKind::InvalidMagic)?
        }

        let version = reader
            .read_u32::<BigEndian>()
            .context(ErrorKind::ReadHeader)?;
        if version != VERSION as u32 {
            Err(ErrorKind::InvalidVersion)?
        }

        let len = reader
            .read_u64::<BigEndian>()
            .context(ErrorKind::ReadHeader)?;
        let start_idx = reader
            .read_u64::<BigEndian>()
            .context(ErrorKind::ReadHeader)?;
        let end_idx = reader
            .read_u64::<BigEndian>()
            .context(ErrorKind::ReadHeader)?;

        let start = LogIndex {
            idx: start_idx,
            len: len as usize,
        };
        let end = LogIndex {
            idx: end_idx,
            len: len as usize,
        };

        let header = Header { len, start, end };
        Ok(header)
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_u32::<BigEndian>(MAGIC)
            .context(ErrorKind::WriteHeader)?;
        writer
            .write_u32::<BigEndian>(VERSION.into())
            .context(ErrorKind::WriteHeader)?;
        writer
            .write_u64::<BigEndian>(self.len)
            .context(ErrorKind::WriteHeader)?;
        writer
            .write_u64::<BigEndian>(self.start.logical() as u64)
            .context(ErrorKind::WriteHeader)?;
        writer
            .write_u64::<BigEndian>(self.end.logical() as u64)
            .context(ErrorKind::WriteHeader)?;
        Ok(())
    }
}

struct LogIndex {
    idx: u64,
    len: usize,
}

impl LogIndex {
    fn logical(&self) -> u64 {
        self.idx
    }

    fn physical(&self) -> usize {
        (self.idx % self.len as u64) as usize
    }
}

impl fmt::Debug for LogIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LogIndex {{ physical: {}, logical: {} }}",
            self.physical(),
            self.logical()
        )
    }
}

impl PartialEq for LogIndex {
    fn eq(&self, other: &Self) -> bool {
        self.logical() == other.logical()
    }
}

impl PartialOrd for LogIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.logical().cmp(&other.logical()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck::{quickcheck, TestResult};
    use std::io::Cursor;

    quickcheck! {
        fn test_logindex(idx: u64, len: usize) -> TestResult {
            if len == 0 {
                TestResult::discard()
            } else {
                let i = LogIndex {
                    idx,
                    len,
                };
                TestResult::from_bool(i.physical() as u64 <= i.logical() && i.physical() < i.len)
            }
        }
    }

    #[test]
    fn test_read() {
        let bytes = [
            0x4du8, 0x4c, 0x4f, 0x47, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x30,
        ];

        let start = LogIndex {
            idx: 32,
            len: 144115188075855888,
        };
        let end = LogIndex {
            idx: 48,
            len: 144115188075855888,
        };
        let expected = Header {
            len: 144115188075855888,
            start,
            end,
        };

        let header = Header::from_read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(expected, header);
    }
}
