use crate::{
    io::{
        formats::KnownFormat, FormatData, KnownSampleType, StreamSpec, SyphonCodec, SyphonFormat,
    },
    signal::{ChannelLayout, Channels, SignalSpecBuilder},
    SyphonError,
};
use std::io::{Read, Write};

const RIFF_CHUNK_ID: &[u8; 4] = b"RIFF";
const WAVE_CHUNK_ID: &[u8; 4] = b"WAVE";

#[derive(Clone, Copy)]
pub struct WaveHeader {
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
    pub avg_byte_rate: u32,
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

impl WaveHeader {
    fn byte_len(&self) -> u32 {
        8 + WAVE_CHUNK_ID.len() as u32
            + 8
            + self.fmt.byte_len()
            + self.fact.as_ref().map_or(0, |f| 8 + f.byte_len())
            + 8
            + self.data.byte_len
    }

    pub fn read(reader: &mut impl Read) -> Result<Self, SyphonError> {
        let mut buf = [0u8; 40];

        reader.read_exact(&mut buf[0..12])?;
        if &buf[0..4] != RIFF_CHUNK_ID || &buf[8..12] != WAVE_CHUNK_ID {
            return Err(SyphonError::InvalidData);
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
                return Err(SyphonError::InvalidData);
            }

            reader.read_exact(&mut buf[..byte_len])?;
            match &chunk_id {
                FMT_CHUNK_ID => {
                    fmt = Some(FmtChunk::read(&buf[..byte_len])?);
                }
                FACT_CHUNK_ID => {
                    fact = Some(FactChunk::read(&buf[..byte_len])?);
                }
                _ => return Err(SyphonError::InvalidData),
            }
        }

        Ok(Self {
            fmt: fmt.ok_or(SyphonError::InvalidData)?,
            fact,
            data,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<(), SyphonError> {
        let mut buf = [0u8; 40];

        buf[0..4].copy_from_slice(RIFF_CHUNK_ID);
        buf[4..8].copy_from_slice(&(self.byte_len() - 8).to_le_bytes());
        buf[8..12].copy_from_slice(WAVE_CHUNK_ID);
        writer.write_all(&buf[0..12])?;

        buf[0..4].copy_from_slice(FMT_CHUNK_ID);
        buf[4..8].copy_from_slice(&self.fmt.byte_len().to_le_bytes());
        let n = self.fmt.write(&mut buf[8..])?;
        writer.write_all(&buf[..n + 8])?;

        if let Some(fact) = &self.fact {
            buf[0..4].copy_from_slice(FACT_CHUNK_ID);
            buf[4..8].copy_from_slice(&fact.byte_len().to_le_bytes());
            let n = fact.write(&mut buf[8..])?;
            writer.write_all(&buf[..n + 8])?;
        }

        buf[0..4].copy_from_slice(DATA_CHUNK_ID);
        buf[4..8].copy_from_slice(&self.data.byte_len.to_le_bytes());
        writer.write_all(&buf[0..8])?;

        Ok(())
    }
}

impl<F> From<WaveHeader> for FormatData<F>
where
    F: KnownFormat,
    SyphonCodec: TryInto<F::Codec>,
{
    fn from(header: WaveHeader) -> Self {
        let codec = match header.fmt.format_tag {
            1 | 3 => Some(SyphonCodec::Pcm),
            _ => None,
        };

        let sample_type = match (header.fmt.format_tag, header.fmt.bits_per_sample) {
            (1, 8) => Some(KnownSampleType::U8),
            (1, 16) => Some(KnownSampleType::I16),
            (1, 32) => Some(KnownSampleType::I32),
            (3, 32) => Some(KnownSampleType::F32),
            (3, 64) => Some(KnownSampleType::F64),
            _ => None,
        };

        let channels = header
            .fmt
            .ext
            .and_then(|ext| Some(ChannelLayout::from_bits(ext.channel_mask).into()))
            .unwrap_or_else(|| Channels::Count(header.fmt.n_channels as u32));

        let decoded_byte_len = header
            .fact
            .zip(sample_type)
            .map(|(fact, s)| fact.n_frames as u64 * channels.count() as u64 * s.byte_size() as u64);

        Self {
            tracks: vec![(
                codec.and_then(|c| c.try_into().ok()),
                StreamSpec {
                    avg_bitrate: todo!(),
                    // compression_ratio: decoded_byte_len
                    //     .map(|decoded| header.data.byte_len as f64 / decoded as f64),
                    sample_type: sample_type.map(Into::into),
                    decoded_spec: SignalSpecBuilder::new()
                        .with_channels(channels)
                        .with_frame_rate(header.fmt.sample_rate)
                        .with_n_frames(header.fact.map(|fact| fact.n_frames as u64)),
                },
            )],
        }
    }
}

impl<F> TryFrom<FormatData<F>> for WaveHeader
where
    F: KnownFormat,
    F::Codec: Copy + PartialEq,
    SyphonCodec: TryInto<F::Codec>
{
    type Error = SyphonError;

    fn try_from(mut data: FormatData<F>) -> Result<Self, Self::Error> {
        if data.tracks.len() != 1 {
            return Err(SyphonError::Unsupported);
        }

        let (codec, spec) = &mut data.tracks[0];
        let expected_codec = SyphonCodec::Pcm.try_into().ok();
        if expected_codec.is_some_and(|c| codec.get_or_insert(c) != &c) {
            return Err(SyphonError::Unsupported);
        }

        let sample_type = spec
            .sample_type
            .ok_or(SyphonError::MissingData)?
            .try_into()?;

        let format_tag = match sample_type {
            KnownSampleType::U8 | KnownSampleType::I16 | KnownSampleType::I32 => 1,
            KnownSampleType::F32 | KnownSampleType::F64 => 3,
            _ => return Err(SyphonError::Unsupported),
        };

        let n_channels = spec
            .decoded_spec
            .channels
            .ok_or(SyphonError::InvalidData)?
            .count() as u16;

        let sample_rate = spec
            .decoded_spec
            .frame_rate
            .ok_or(SyphonError::InvalidData)?;

        Ok(Self {
            fmt: FmtChunk {
                format_tag,
                n_channels,
                sample_rate,
                avg_byte_rate: sample_rate * sample_type.byte_size() as u32,
                block_align: sample_type.byte_size() as u16 * n_channels,
                bits_per_sample: sample_type.byte_size() as u16 * 8,
                ext: None,
            },
            fact: spec
                .decoded_spec
                .n_frames
                .map(|n| FactChunk { n_frames: n as u32 }),
            data: DataChunk {
                byte_len: spec.n_bytes().ok_or(SyphonError::Unsupported)? as u32,
            },
        })
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
            return Err(SyphonError::InvalidData);
        }

        let mut chunk = Self {
            format_tag: u16::from_le_bytes(buf[0..2].try_into().unwrap()),
            n_channels: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
            sample_rate: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            avg_byte_rate: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
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
            return Err(SyphonError::InvalidData);
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
            return Err(SyphonError::InvalidData);
        }

        buf[0..2].copy_from_slice(&self.format_tag.to_le_bytes());
        buf[2..4].copy_from_slice(&self.n_channels.to_le_bytes());
        buf[4..8].copy_from_slice(&self.sample_rate.to_le_bytes());
        buf[8..12].copy_from_slice(&self.avg_byte_rate.to_le_bytes());
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
            return Err(SyphonError::InvalidData);
        }

        Ok(Self {
            n_frames: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
        })
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        if buf.len() < 4 {
            return Err(SyphonError::InvalidData);
        }

        buf[0..4].copy_from_slice(&self.n_frames.to_le_bytes());
        Ok(4)
    }
}
