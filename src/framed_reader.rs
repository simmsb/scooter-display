use deku::{no_std_io, writer::Writer};
use embedded_io_async::Read;

#[derive(defmt::Format)]
pub enum ReadFramedError {
    ReadError,
    BufferTooSmall,
    CrcMismatch,
}

#[derive(defmt::Format)]
pub enum WriteFramedError {
    NumericOverflow,
    BufferTooSmall,
}

pub async fn read_framed<'buf, R: Read>(
    reader: &mut R,
    start_byte: u8,
    buf: &'buf mut [u8],
) -> Result<&'buf [u8], ReadFramedError> {
    let mut start_byte_buf = [0u8; 1];
    loop {
        reader
            .read_exact(&mut start_byte_buf)
            .await
            .map_err(|_| ReadFramedError::ReadError)?;

        if start_byte_buf[0] == start_byte {
            break;
        }
    }

    let mut len_buf = [0u8; 1];
    reader
        .read_exact(&mut len_buf)
        .await
        .map_err(|_| ReadFramedError::ReadError)?;

    // total_len includes start byte, length, and trailing CRC
    let total_len = len_buf[0];
    let len = total_len.saturating_sub(4) as usize;
    if len > buf.len() {
        return Err(ReadFramedError::BufferTooSmall);
    }

    reader
        .read_exact(&mut buf[0..len])
        .await
        .map_err(|_| ReadFramedError::ReadError)?;
    let mut received_crc = [0u8; 2];
    reader
        .read_exact(&mut received_crc)
        .await
        .map_err(|_| ReadFramedError::ReadError)?;

    let crc = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
    let mut digest = crc.digest();
    digest.update(&[start_byte, total_len]);
    digest.update(&buf[0..len]);
    let calculated_crc = digest.finalize();
    let received_crc = u16::from_le_bytes(received_crc);

    if calculated_crc != received_crc {
        defmt::warn!(
            "CRC mismatch, got {} expected {}",
            received_crc,
            calculated_crc
        );
    }

    Ok(&buf[0..len])
}

pub fn assemble_framed(
    buf: &mut [u8],
    start_byte: u8,
    body: &[u8],
) -> Result<(), WriteFramedError> {
    let total_len: u8 = body
        .len()
        .checked_add(4)
        .and_then(|n| n.try_into().ok())
        .ok_or(WriteFramedError::NumericOverflow)?;

    if buf.len() < total_len as usize {
        return Err(WriteFramedError::BufferTooSmall);
    }

    buf[0] = start_byte;
    buf[1] = total_len;
    buf[2..(body.len() + 2)].copy_from_slice(body);

    let crc = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
    let mut digest = crc.digest();
    digest.update(&buf[0..(total_len as usize - 2)]);
    let calculated_crc = digest.finalize().to_le_bytes();

    buf[(total_len as usize - 2)..(total_len as usize)].copy_from_slice(&calculated_crc);

    Ok(())
}

pub const fn buffer_size_for_type<T: deku::DekuSize>() -> usize {
    T::SIZE_BITS.div_ceil(8) + 4
}

pub fn assemble_framed_deku<'a, T: deku::DekuWriter + deku::DekuSize>(
    buf: &'a mut [u8],
    start_byte: u8,
    body: &T,
) -> Result<&'a [u8], WriteFramedError> {
    let overhead_len = 4;

    let required_buffer_len: u8 = T::SIZE_BYTES
        .unwrap()
        .checked_add(overhead_len)
        .and_then(|n| n.try_into().ok())
        .ok_or(WriteFramedError::NumericOverflow)?;

    if buf.len() < required_buffer_len as usize {
        return Err(WriteFramedError::BufferTooSmall);
    }

    buf[0] = start_byte;

    let mut cursor = no_std_io::Cursor::new(&mut buf[2..]);
    let mut writer = Writer::new(&mut cursor);
    body.to_writer(&mut writer, ()).unwrap();
    let _ = writer.finalize();
    let body_len = writer.bits_written.div_ceil(8);

    // we know this won't overflow from the checked_add above
    let total_len = body_len + overhead_len;

    buf[1] = total_len as u8;

    let crc = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
    let mut digest = crc.digest();
    digest.update(&buf[0..(total_len as usize - 2)]);
    let calculated_crc = digest.finalize().to_le_bytes();

    buf[(total_len as usize - 2)..(total_len as usize)].copy_from_slice(&calculated_crc);

    Ok(&buf[..(total_len as usize)])
}
