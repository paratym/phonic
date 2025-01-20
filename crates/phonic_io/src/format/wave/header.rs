use crate::{
    format::wave::{WaveFormatTag, WaveSupportedCodec},
    CodecTag, FormatTag, StreamSpec, TypeLayout,
};
use phonic_signal::{
    ChannelLayout, Channels, PhonicError, PhonicResult, SignalSpec, SignalSpecBuilder,
};
use std::{
    any::TypeId,
    io::{Read, Write},
};

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
    pub fn byte_len(&self) -> u32 {
        8 + WAVE_CHUNK_ID.len() as u32
            + 8
            + self.fmt.byte_len()
            + self.fact.as_ref().map_or(0, |f| 8 + f.byte_len())
            + 8
            + self.data.byte_len
    }

    pub fn read(reader: &mut impl Read) -> PhonicResult<Self> {
        let mut buf = [0u8; 40];

        reader.read_exact(&mut buf[0..12])?;
        if &buf[0..4] != RIFF_CHUNK_ID || &buf[8..12] != WAVE_CHUNK_ID {
            return Err(PhonicError::InvalidData);
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
                return Err(PhonicError::InvalidData);
            }

            reader.read_exact(&mut buf[..byte_len])?;
            match &chunk_id {
                FMT_CHUNK_ID => {
                    fmt = Some(FmtChunk::read(&buf[..byte_len])?);
                }
                FACT_CHUNK_ID => {
                    fact = Some(FactChunk::read(&buf[..byte_len])?);
                }
                _ => return Err(PhonicError::InvalidData),
            }
        }

        Ok(Self {
            fmt: fmt.ok_or(PhonicError::InvalidData)?,
            fact,
            data,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> PhonicResult<()> {
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

impl<C> TryFrom<WaveHeader> for StreamSpec<C>
where
    C: CodecTag,
    WaveSupportedCodec: TryInto<C>,
    PhonicError: From<<WaveSupportedCodec as TryInto<C>>::Error>,
{
    type Error = PhonicError;

    fn try_from(header: WaveHeader) -> Result<Self, Self::Error> {
        let codec = match header.fmt.format_tag {
            1 | 3 => WaveSupportedCodec::PcmLE.try_into(),
            _ => return Err(PhonicError::Unsupported),
        }?;

        let sample_layout = match (header.fmt.format_tag, header.fmt.bits_per_sample) {
            (1, 8) => TypeLayout::of::<u8>(),
            (1, 16) => TypeLayout::of::<i16>(),
            (1, 32) => TypeLayout::of::<i32>(),
            (3, 32) => TypeLayout::of::<f32>(),
            (3, 64) => TypeLayout::of::<f64>(),
            _ => return Err(PhonicError::Unsupported),
        };

        let channels = match header.fmt.ext {
            Some(ext) => ChannelLayout::from_mask(ext.channel_mask).into(),
            None => Channels::Count(header.fmt.n_channels as u32),
        };

        Ok(Self {
            codec,
            avg_byte_rate: header.fmt.avg_byte_rate,
            block_align: header.fmt.block_align as usize,
            sample_layout,
            decoded_spec: SignalSpec {
                sample_rate: header.fmt.sample_rate,
                channels,
            },
        })
    }
}

impl<C> TryFrom<StreamSpec<C>> for WaveHeader
where
    C: CodecTag + TryInto<WaveSupportedCodec>,
    PhonicError: From<<C as TryInto<WaveSupportedCodec>>::Error>,
{
    type Error = PhonicError;

    fn try_from(spec: StreamSpec<C>) -> Result<Self, Self::Error> {
        todo!()
        // let codec = spec.codec.try_into()?;
        // let format_tag = match (codec, spec.sample_type) {
        //     (WaveSupportedCodec::Pcm, sample_type)
        //         if sample_type == TypeId::of::<u8>()
        //             || sample_type == TypeId::of::<i16>()
        //             || sample_type == TypeId::of::<i32>() =>
        //     {
        //         1
        //     }
        //     (WaveSupportedCodec::Pcm, sample_type)
        //         if sample_type == TypeId::of::<f32>() || sample_type == TypeId::of::<f64>() =>
        //     {
        //         3
        //     }
        //     _ => return Err(PhonicError::Unsupported),
        // };
        //
        // Ok(Self {
        //     fmt: FmtChunk {
        //         format_tag,
        //         n_channels: spec.decoded_spec.channels.count() as u16,
        //         sample_rate: spec.decoded_spec.sample_rate,
        //         avg_byte_rate: spec.avg_byte_rate,
        //         block_align: spec.block_align as u16,
        //         bits_per_sample: sample_type.byte_size() as u16 * 8,
        //         ext: None,
        //     },
        //     fact: None, // TODO: Some(FactChunk { n_frames: todo!() }),
        //     data: DataChunk { byte_len: 0 }, // TODO
        // })
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

    fn read(buf: &[u8]) -> PhonicResult<Self> {
        let buf_len = buf.len();
        if buf_len != 16 && buf_len != 18 && buf_len != 40 {
            return Err(PhonicError::InvalidData);
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
            return Err(PhonicError::InvalidData);
        }

        chunk.ext = Some(FmtChunkExt {
            valid_bits_per_sample: u16::from_le_bytes(buf[18..20].try_into().unwrap()),
            channel_mask: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            sub_format: buf[24..40].try_into().unwrap(),
        });

        Ok(chunk)
    }

    fn write(&self, buf: &mut [u8]) -> PhonicResult<usize> {
        let byte_len = self.byte_len() as usize;
        if buf.len() < byte_len {
            return Err(PhonicError::InvalidData);
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

    fn read(buf: &[u8]) -> PhonicResult<Self> {
        let buf_len = buf.len();
        if buf_len != 4 {
            return Err(PhonicError::InvalidData);
        }

        Ok(Self {
            n_frames: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
        })
    }

    fn write(&self, buf: &mut [u8]) -> PhonicResult<usize> {
        if buf.len() < 4 {
            return Err(PhonicError::InvalidData);
        }

        buf[0..4].copy_from_slice(&self.n_frames.to_le_bytes());
        Ok(4)
    }
}
