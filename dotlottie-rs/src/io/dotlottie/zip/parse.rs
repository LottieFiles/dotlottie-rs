use std::fmt;

const EOCD_SIGNATURE: u32 = 0x06054b50;
const CENTRAL_DIR_SIGNATURE: u32 = 0x02014b50;
const LOCAL_FILE_HEADER_SIGNATURE: u32 = 0x04034b50;

const EOCD_MIN_SIZE: usize = 22;
const CENTRAL_DIR_ENTRY_MIN_SIZE: usize = 46;
const LOCAL_FILE_HEADER_MIN_SIZE: usize = 30;

// 64KB max comment + 22-byte EOCD
const EOCD_MAX_SEARCH: usize = 65557;

#[derive(Debug)]
pub enum ZipError {
    TooSmall,
    EocdNotFound,
    InvalidCentralDir,
    InvalidLocalHeader,
    UnsupportedCompression(u16),
    DecompressError,
    FileNotFound,
    OutOfBounds,
}

impl fmt::Display for ZipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ZipError {}

#[derive(Debug, Clone)]
pub(crate) struct CentralDirEntry {
    pub name: Box<str>,
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub local_header_offset: u32,
    pub compression_method: u16,
}

struct Eocd {
    central_dir_offset: u32,
    total_entries: u16,
}

#[inline]
fn read_u16(data: &[u8], offset: usize) -> Result<u16, ZipError> {
    let end = offset.checked_add(2).ok_or(ZipError::OutOfBounds)?;
    data.get(offset..end)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or(ZipError::OutOfBounds)
}

#[inline]
fn read_u32(data: &[u8], offset: usize) -> Result<u32, ZipError> {
    let end = offset.checked_add(4).ok_or(ZipError::OutOfBounds)?;
    data.get(offset..end)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or(ZipError::OutOfBounds)
}

fn find_eocd(data: &[u8]) -> Result<Eocd, ZipError> {
    if data.len() < EOCD_MIN_SIZE {
        return Err(ZipError::TooSmall);
    }

    let search_start = data.len().saturating_sub(EOCD_MAX_SEARCH);

    let mut pos = data.len() - EOCD_MIN_SIZE;
    loop {
        if read_u32(data, pos)? == EOCD_SIGNATURE {
            let total_entries = read_u16(data, pos + 10)?;
            let central_dir_offset = read_u32(data, pos + 16)?;

            if central_dir_offset as usize > data.len() {
                return Err(ZipError::InvalidCentralDir);
            }

            return Ok(Eocd {
                central_dir_offset,
                total_entries,
            });
        }
        if pos == search_start {
            break;
        }
        pos -= 1;
    }

    Err(ZipError::EocdNotFound)
}

fn parse_central_directory(data: &[u8], eocd: &Eocd) -> Result<Vec<CentralDirEntry>, ZipError> {
    let mut entries = Vec::with_capacity(eocd.total_entries as usize);
    let mut offset = eocd.central_dir_offset as usize;

    for _ in 0..eocd.total_entries {
        if offset
            .checked_add(CENTRAL_DIR_ENTRY_MIN_SIZE)
            .is_none_or(|end| end > data.len())
        {
            return Err(ZipError::InvalidCentralDir);
        }

        if read_u32(data, offset)? != CENTRAL_DIR_SIGNATURE {
            return Err(ZipError::InvalidCentralDir);
        }

        let compression_method = read_u16(data, offset + 10)?;
        let crc32 = read_u32(data, offset + 16)?;
        let compressed_size = read_u32(data, offset + 20)?;
        let uncompressed_size = read_u32(data, offset + 24)?;
        let file_name_len = read_u16(data, offset + 28)? as usize;
        let extra_field_len = read_u16(data, offset + 30)? as usize;
        let file_comment_len = read_u16(data, offset + 32)? as usize;
        let local_header_offset = read_u32(data, offset + 42)?;

        let name_start = offset
            .checked_add(CENTRAL_DIR_ENTRY_MIN_SIZE)
            .ok_or(ZipError::InvalidCentralDir)?;
        let name_end = name_start
            .checked_add(file_name_len)
            .ok_or(ZipError::InvalidCentralDir)?;
        if name_end > data.len() {
            return Err(ZipError::InvalidCentralDir);
        }

        let name = std::str::from_utf8(&data[name_start..name_end])
            .map_err(|_| ZipError::InvalidCentralDir)?;

        // Skip directory entries (names ending with '/')
        if !name.ends_with('/') {
            entries.push(CentralDirEntry {
                name: Box::from(name),
                compression_method,
                crc32,
                compressed_size,
                uncompressed_size,
                local_header_offset,
            });
        }

        offset = name_end
            .checked_add(extra_field_len)
            .and_then(|o| o.checked_add(file_comment_len))
            .ok_or(ZipError::InvalidCentralDir)?;

        if offset > data.len() {
            return Err(ZipError::InvalidCentralDir);
        }
    }

    entries.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

pub(crate) fn locate_file_data(
    data: &[u8],
    entry: &CentralDirEntry,
) -> Result<(usize, usize), ZipError> {
    let offset = entry.local_header_offset as usize;

    if offset
        .checked_add(LOCAL_FILE_HEADER_MIN_SIZE)
        .is_none_or(|end| end > data.len())
    {
        return Err(ZipError::InvalidLocalHeader);
    }

    if read_u32(data, offset)? != LOCAL_FILE_HEADER_SIGNATURE {
        return Err(ZipError::InvalidLocalHeader);
    }

    let file_name_len = read_u16(data, offset + 26)? as usize;
    let extra_field_len = read_u16(data, offset + 28)? as usize;

    let data_offset = offset
        .checked_add(LOCAL_FILE_HEADER_MIN_SIZE)
        .and_then(|o| o.checked_add(file_name_len))
        .and_then(|o| o.checked_add(extra_field_len))
        .ok_or(ZipError::InvalidLocalHeader)?;

    let data_len = entry.compressed_size as usize;

    let end = data_offset
        .checked_add(data_len)
        .ok_or(ZipError::OutOfBounds)?;

    if end > data.len() {
        return Err(ZipError::OutOfBounds);
    }

    Ok((data_offset, data_len))
}

pub(crate) fn parse_archive(data: &[u8]) -> Result<Vec<CentralDirEntry>, ZipError> {
    let eocd = find_eocd(data)?;
    parse_central_directory(data, &eocd)
}
