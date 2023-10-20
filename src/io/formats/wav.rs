use crate::{
    io::{
        EncodedStreamSpecBuilder, FormatIdentifiers, FormatReadResult, FormatReader, MediaSource,
        StreamSpec, StreamSpecBuilder, SyphonCodec, EncodedStreamReader,
    },
    SampleFormat, SyphonError,
};
use std::io::{Read, Seek, SeekFrom, Write};

// implementation based on:
// https://www.mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html

pub static WAV_FORMAT_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[
        RIFF_CHUNK_ID,
        WAV_CHUNK_ID,
        FMT_CHUNK_ID,
        FACT_CHUNK_ID,
        DATA_CHUNK_ID,
    ],
};

const RIFF_CHUNK_ID: &[u8; 4] = b"RIFF";
const WAV_CHUNK_ID: &[u8; 4] = b"WAVE";

#[derive(Clone, Copy)]
pub struct WavHeader {
    pub fmt: FmtChunk,
    pub fact: Option<FactChunk>,
    pub data: DataChunk,
}

const FMT_CHUNK_ID: &[u8; 4] = b"fmt ";

#[derive(Clone, Copy)]
pub struct FmtChunk {
    pub format_tag: u16,
    pub n_channels: u16,
    pub sample_rate: u32,
    pub avg_bytes_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub ext: Option<FmtChunkExt>,
}

#[derive(Clone, Copy)]
pub struct FmtChunkExt {
    pub valid_bits_per_sample: u16,
    pub channel_mask: u32,
    pub sub_format: [u8; 16],
}

const FACT_CHUNK_ID: &[u8; 4] = b"fact";

#[derive(Clone, Copy)]
pub struct FactChunk {
    pub n_frames: u32,
}

const DATA_CHUNK_ID: &[u8; 4] = b"data";

#[derive(Clone, Copy)]
pub struct DataChunk {
    pub byte_len: u32,
}

impl WavHeader {
    fn byte_len(&self) -> u32 {
        8 + WAV_CHUNK_ID.len() as u32
            + 8
            + self.fmt.byte_len()
            + self.fact.as_ref().map_or(0, |f| 8 + f.byte_len())
            + 8
            + self.data.byte_len
    }

    fn read(reader: &mut impl Read) -> Result<Self, SyphonError> {
        let mut buf = [0u8; 40];

        reader.read_exact(&mut buf[0..12])?;
        if &buf[0..4] != RIFF_CHUNK_ID || &buf[8..12] != WAV_CHUNK_ID {
            return Err(SyphonError::MalformedData);
        }

        let mut fmt = None;
        let mut fact = None;
        let data;

        loop {
            reader.read_exact(&mut buf[..8])?;
            let chunk_id: [u8; 4] = buf[0..4].try_into().unwrap();
            let byte_len = u32::from_le_bytes(buf[4..8].try_into().unwrap());

            if &chunk_id == DATA_CHUNK_ID {
                data = DataChunk { byte_len };
                break;
            }

            let byte_len = byte_len as usize;
            if byte_len > buf.len() {
                return Err(SyphonError::MalformedData);
            }

            match &chunk_id {
                FMT_CHUNK_ID => {
                    fmt = Some(FmtChunk::read(&buf[..byte_len])?);
                }
                FACT_CHUNK_ID => {
                    fact = Some(FactChunk::read(&buf[..byte_len])?);
                }
                _ => return Err(SyphonError::MalformedData),
            }
        }

        Ok(Self {
            fmt: fmt.ok_or(SyphonError::MalformedData)?,
            fact,
            data,
        })
    }

    fn write(&self, writer: &mut impl Write) -> Result<(), SyphonError> {
        let mut buf = [0u8; 40];

        buf[0..4].copy_from_slice(RIFF_CHUNK_ID);
        buf[4..8].copy_from_slice(&(self.byte_len() - 8).to_le_bytes());
        buf[8..12].copy_from_slice(WAV_CHUNK_ID);
        writer.write_all(&buf[0..12])?;

        buf[0..4].copy_from_slice(FMT_CHUNK_ID);
        buf[4..8].copy_from_slice(&self.fmt.byte_len().to_le_bytes());
        let mut n = self.fmt.write(&mut buf[8..])?;
        writer.write_all(&buf[..n + 8])?;

        if let Some(fact) = &self.fact {
            buf[0..4].copy_from_slice(FACT_CHUNK_ID);
            buf[4..8].copy_from_slice(&fact.byte_len().to_le_bytes());
            n = fact.write(&mut buf)?;
            writer.write_all(&buf[..n + 8])?;
        }

        buf[0..4].copy_from_slice(DATA_CHUNK_ID);
        buf[4..8].copy_from_slice(&self.data.byte_len.to_le_bytes());
        writer.write_all(&buf[0..8])?;

        Ok(())
    }
}

impl From<WavHeader> for EncodedStreamSpecBuilder {
    fn from(header: WavHeader) -> Self {
        let codec_key = match header.fmt.format_tag {
            1 | 3 => Some(SyphonCodec::Pcm),
            _ => None,
        };

        let sample_format = match (header.fmt.format_tag, header.fmt.bits_per_sample) {
            (1, 8) => Some(SampleFormat::U8),
            (1, 16) => Some(SampleFormat::I16),
            (1, 32) => Some(SampleFormat::I32),
            (3, 32) => Some(SampleFormat::F32),
            (3, 64) => Some(SampleFormat::F64),
            _ => None,
        };

        Self {
            codec_key,
            block_size: Some(header.fmt.block_align as usize),
            byte_len: Some(header.data.byte_len as u64),
            decoded_spec: StreamSpecBuilder {
                sample_format,
                n_channels: Some(header.fmt.n_channels as u8),
                n_frames: header.fact.map(|fact| fact.n_frames as u64),
                sample_rate: Some(header.fmt.sample_rate),
                block_size: None,
            },
        }
    }
}

impl FmtChunk {
    fn byte_len(&self) -> u32 {
        if self.ext.is_some() {
            40
        } else {
            16
        }
    }

    fn read(buf: &[u8]) -> Result<Self, SyphonError> {
        let buf_len = buf.len();
        if buf_len != 16 && buf_len != 18 && buf_len != 40 {
            return Err(SyphonError::MalformedData);
        }

        let mut chunk = Self {
            format_tag: u16::from_le_bytes(buf[0..2].try_into().unwrap()),
            n_channels: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
            sample_rate: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            avg_bytes_rate: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            block_align: u16::from_le_bytes(buf[12..14].try_into().unwrap()),
            bits_per_sample: u16::from_le_bytes(buf[14..16].try_into().unwrap()),
            ext: None,
        };

        if buf_len < 18 {
            return Ok(chunk);
        }

        let ext_len = u16::from_le_bytes(buf[16..18].try_into().unwrap());
        if ext_len == 0 && buf_len == 18 {
            return Ok(chunk);
        } else if ext_len != 22 && buf_len != 40 {
            return Err(SyphonError::MalformedData);
        }

        chunk.ext = Some(FmtChunkExt {
            valid_bits_per_sample: u16::from_le_bytes(buf[18..20].try_into().unwrap()),
            channel_mask: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            sub_format: buf[24..40].try_into().unwrap(),
        });

        Ok(chunk)
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        let byte_len = self.byte_len() as usize;
        if buf.len() < byte_len {
            return Err(SyphonError::BadRequest);
        }

        buf[0..2].copy_from_slice(&self.format_tag.to_le_bytes());
        buf[2..4].copy_from_slice(&self.n_channels.to_le_bytes());
        buf[4..8].copy_from_slice(&self.sample_rate.to_le_bytes());
        buf[8..12].copy_from_slice(&self.avg_bytes_rate.to_le_bytes());
        buf[12..14].copy_from_slice(&self.block_align.to_le_bytes());
        buf[14..16].copy_from_slice(&self.bits_per_sample.to_le_bytes());

        if let Some(ext) = &self.ext {
            buf[16..18].copy_from_slice(&22u16.to_le_bytes());
            buf[18..20].copy_from_slice(&ext.valid_bits_per_sample.to_le_bytes());
            buf[20..24].copy_from_slice(&ext.channel_mask.to_le_bytes());
            buf[24..40].copy_from_slice(&ext.sub_format);
        }

        Ok(byte_len)
    }
}

impl FactChunk {
    fn byte_len(&self) -> u32 {
        4
    }

    fn read(buf: &[u8]) -> Result<Self, SyphonError> {
        let buf_len = buf.len();
        if buf_len != 4 {
            return Err(SyphonError::MalformedData);
        }

        Ok(Self {
            n_frames: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
        })
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        if buf.len() < 4 {
            return Err(SyphonError::BadRequest);
        }

        buf[0..4].copy_from_slice(&self.n_frames.to_le_bytes());
        Ok(4)
    }
}

pub struct WavFormat<T> {
    header: WavHeader,
    inner: T,
    i: usize,
}

impl<T: Read> WavFormat<T> {
    pub fn reader(mut inner: T) -> std::io::Result<Self> {
        let header = WavHeader::read(&mut inner)?;

        Ok(Self {
            header,
            inner,
            i: 0,
        })
    }
}

impl<T: Read> Read for WavFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut buf_len = buf.len().min(self.header.data.byte_len as usize - self.i);
        buf_len -= buf_len % self.header.fmt.block_align as usize;

        let n = self.inner.read(&mut buf[..buf_len])?;
        self.i += n;

        if n % self.header.fmt.block_align as usize != 0 {
            todo!();
        }

        Ok(n)
    }
}

impl<T: Write> WavFormat<T> {
    pub fn writer(mut inner: T, header: WavHeader) -> std::io::Result<Self> {
        header.write(&mut inner)?;

        Ok(Self {
            header,
            inner,
            i: 0,
        })
    }
}

impl<T: Write> Write for WavFormat<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buf_len = buf.len().min(self.header.data.byte_len as usize - self.i);
        buf_len -= buf_len % self.header.fmt.block_align as usize;

        let n = self.inner.write(&buf[..buf_len])?;
        self.i += n;

        if n % self.header.fmt.block_align as usize != 0 {
            todo!();
        }

        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for WavFormat<T> {
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        todo!()

        // let new_pos = match offset {
        //     SeekFrom::Current(offset) if offset == 0 => {
        //         return Ok(self.media_pos());
        //     }
        //     SeekFrom::Current(offset) => self.source_pos as i64 + offset,
        //     SeekFrom::Start(offset) => self.media_bounds.0 as i64 + offset as i64,
        //     SeekFrom::End(offset) => self.media_bounds.1 as i64 + offset,
        // };

        // if new_pos < 0 {
        //     return Err(SyphonError::BadRequest);
        // }

        // let mut new_pos = new_pos as u64;
        // if new_pos < self.media_bounds.0 || new_pos >= self.media_bounds.1 {
        //     return Err(SyphonError::BadRequest);
        // }

        // let new_media_pos = new_pos - self.media_bounds.0;
        // let block_size = self.stream_spec_mut().block_size.unwrap_or(1);
        // new_pos -= new_media_pos % block_size as u64;

        // self.source_pos = self.source.seek(SeekFrom::Start(new_pos))?;
        // Ok(self.media_pos())
    }
}

impl<T: Read + Seek> WavFormat<T> {
    pub fn into_format_reader(self) -> WavFormatReader<T> {
        WavFormatReader::from(self)
    }
}

pub struct WavFormatReader<T> {
    inner: WavFormat<T>,
    tracks: [EncodedStreamSpecBuilder; 1],
}

impl<T> From<WavFormat<T>> for WavFormatReader<T> {
    fn from(inner: WavFormat<T>) -> Self {
        let tracks = [inner.header.into()];
        Self { inner, tracks }
    }
}

impl<T: Read + Seek> FormatReader for WavFormatReader<T> {
    fn tracks(&self) -> &[EncodedStreamSpecBuilder] {
        &self.tracks
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        let n = self.inner.read(buf)?;
        Ok(FormatReadResult { track_i: 0, n })
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        Ok(self.inner.seek(offset)?)
    }
}