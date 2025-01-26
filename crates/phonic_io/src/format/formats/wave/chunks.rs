// https://www.mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html
// https://ccrma.stanford.edu/courses/422-winter-2014/projects/WaveFormat/
// https://tech.ebu.ch/docs/tech/tech3285.pdf
// https://github.com/tpn/winsdk-10/blob/master/Include/10.0.14393.0/shared/mmreg.h
// https://datatracker.ietf.org/doc/html/rfc2361

use crate::{
    formats::wave::{RiffChunk, WaveSupportedCodec},
    CodecTag, StreamSpec, StreamSpecBuilder, TypeLayout,
};
use phonic_signal::{PhonicError, PhonicResult, SignalSpec};
use std::io::{self, ErrorKind, Read, Seek, Write};

pub(super) struct FmtChunk {
    w_format_tag: u16,
    n_channels: u16,
    n_samples_per_sec: u32,
    n_avg_bytes_per_sec: u32,
    n_block_align: u16,
    w_bits_per_sample: u16,
    extension: Option<FmtExt>,
}

struct FmtExt {
    w_valid_bits_per_sample: u16,
    dw_channel_mask: u32,
    sub_format: [u8; 16],
}

pub(super) struct FactChunk {
    dw_sample_length: u32,
}

impl FmtChunk {
    pub const CHUNK_ID: [u8; 4] = *b"fmt ";

    pub fn apply_to_spec<C>(self, spec: &mut StreamSpecBuilder<C>) -> PhonicResult<()>
    where
        C: CodecTag,
        WaveSupportedCodec: TryInto<C>,
        PhonicError: From<<WaveSupportedCodec as TryInto<C>>::Error>,
    {
        let Self {
            w_format_tag,
            n_channels,
            n_samples_per_sec,
            n_avg_bytes_per_sec,
            n_block_align,
            w_bits_per_sample,
            extension,
            ..
        } = self;

        let (codec, sample_layout) = match (w_format_tag, w_bits_per_sample) {
            (0x0001, 8) => Some((WaveSupportedCodec::PcmLE, TypeLayout::of::<u8>())).unzip(),
            (0x0001, 16) => Some((WaveSupportedCodec::PcmLE, TypeLayout::of::<i16>())).unzip(),
            (0x0001, 32) => Some((WaveSupportedCodec::PcmLE, TypeLayout::of::<i32>())).unzip(),
            (0x0003, 32) => Some((WaveSupportedCodec::PcmLE, TypeLayout::of::<f32>())).unzip(),
            (0x0003, 64) => Some((WaveSupportedCodec::PcmLE, TypeLayout::of::<f64>())).unzip(),
            _ => (None, None),
        };

        spec.codec = codec.map(TryInto::try_into).transpose()?;
        spec.sample_layout = sample_layout;

        spec.decoded_spec = SignalSpec::builder()
            .with_channels(n_channels as u32)
            .with_sample_rate(n_samples_per_sec);

        spec.avg_byte_rate = Some(n_avg_bytes_per_sec);
        spec.block_align = Some(n_block_align as usize);

        Ok(())
    }

    fn read_inner(reader: &mut impl Read) -> io::Result<Self> {
        let w_format_tag = read_u16(reader)?;
        let n_channels = read_u16(reader)?;
        let n_samples_per_sec = read_u32(reader)?;
        let n_avg_bytes_per_sec = read_u32(reader)?;
        let n_block_align = read_u16(reader)?;
        let w_bits_per_sample = read_u16(reader)?;

        let cb_size = match read_u16(reader) {
            Ok(size) => Some(size),
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => None,
            Err(e) => return Err(e),
        };

        let extension = if let Some(cb_size) = cb_size {
            const EXPECTED_EXT_LEN: u16 = 22;
            if cb_size != EXPECTED_EXT_LEN {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "invalid format chunk extension length",
                ));
            }

            Some(FmtExt::read(reader)?)
        } else {
            None
        };

        Ok(Self {
            w_format_tag,
            n_channels,
            n_samples_per_sec,
            n_avg_bytes_per_sec,
            n_block_align,
            w_bits_per_sample,
            extension,
        })
    }

    pub fn read_riff_chunk(chunk: &mut RiffChunk<impl Read>) -> io::Result<Self> {
        // TODO: should these be enforced
        debug_assert_eq!(chunk.id(), Self::CHUNK_ID);
        debug_assert!(matches!(chunk.len(), 16 | 18 | 40));
        debug_assert_eq!(chunk.pos(), 0);

        let fmt_chunk = Self::read_inner(chunk)?;
        debug_assert_eq!(chunk.pos(), chunk.len());

        Ok(fmt_chunk)
    }

    pub fn try_from_spec<C>(spec: StreamSpec<C>) -> PhonicResult<Self>
    where
        C: CodecTag + TryInto<WaveSupportedCodec>,
        PhonicError: From<<C as TryInto<WaveSupportedCodec>>::Error>,
    {
        let StreamSpec {
            codec,
            avg_byte_rate,
            block_align,
            sample_layout,
            decoded_spec,
        } = spec;

        let SignalSpec {
            sample_rate,
            channels,
        } = decoded_spec;

        let native_codec = codec.try_into()?;
        let w_format_tag = match native_codec {
            WaveSupportedCodec::PcmLE if sample_layout.is::<u8>() => 0x0001,
            WaveSupportedCodec::PcmLE if sample_layout.is::<i16>() => 0x0001,
            WaveSupportedCodec::PcmLE if sample_layout.is::<i32>() => 0x0001,
            WaveSupportedCodec::PcmLE if sample_layout.is::<f32>() => 0x0003,
            WaveSupportedCodec::PcmLE if sample_layout.is::<f64>() => 0x0003,
            _ => return Err(PhonicError::unsupported()),
        };

        Ok(Self {
            w_format_tag,
            n_channels: channels.count() as u16,
            n_samples_per_sec: sample_rate,
            n_avg_bytes_per_sec: avg_byte_rate,
            n_block_align: block_align as u16,
            w_bits_per_sample: sample_layout.size() as u16 * 8,
            extension: None,
        })
    }

    fn write_inner(self, writer: &mut impl Write) -> io::Result<()> {
        let Self {
            w_format_tag,
            n_channels,
            n_samples_per_sec,
            n_avg_bytes_per_sec,
            n_block_align,
            w_bits_per_sample,
            extension,
        } = self;

        writer.write_all(&w_format_tag.to_le_bytes())?;
        writer.write_all(&n_channels.to_le_bytes())?;
        writer.write_all(&n_samples_per_sec.to_le_bytes())?;
        writer.write_all(&n_avg_bytes_per_sec.to_le_bytes())?;
        writer.write_all(&n_block_align.to_le_bytes())?;
        writer.write_all(&w_bits_per_sample.to_le_bytes())?;

        if let Some(extension) = extension {
            extension.write(writer)?
        }

        Ok(())
    }

    pub fn write_riff_chunk<W: Write + Seek>(self, writer: &mut W) -> io::Result<()> {
        let mut chunk = RiffChunk::write_new(writer, Self::CHUNK_ID)?;

        self.write_inner(&mut chunk)?;
        chunk.update_header()?;

        debug_assert!(matches!(chunk.len(), 16 | 18 | 40));
        debug_assert_eq!(chunk.pos(), chunk.len());

        Ok(())
    }
}

impl FmtExt {
    fn read(reader: &mut impl Read) -> io::Result<Self> {
        let w_valid_bits_per_sample = read_u16(reader)?;
        let dw_channel_mask = read_u32(reader)?;

        let mut sub_format = [0u8; 16];
        reader.read_exact(&mut sub_format)?;

        Ok(Self {
            w_valid_bits_per_sample,
            dw_channel_mask,
            sub_format,
        })
    }

    fn write(self, writer: &mut impl Write) -> io::Result<()> {
        let Self {
            w_valid_bits_per_sample,
            dw_channel_mask,
            sub_format,
        } = self;

        writer.write_all(&w_valid_bits_per_sample.to_le_bytes())?;
        writer.write_all(&dw_channel_mask.to_le_bytes())?;
        writer.write_all(&sub_format)?;

        Ok(())
    }
}

impl FactChunk {
    pub const CHUNK_ID: [u8; 4] = *b"fact";

    fn read_inner(reader: &mut impl Read) -> io::Result<Self> {
        let dw_sample_length = read_u32(reader)?;
        Ok(Self { dw_sample_length })
    }

    pub fn read_riff_chunk(chunk: &mut RiffChunk<impl Read>) -> io::Result<Self> {
        // TODO: should these be enforced
        debug_assert_eq!(chunk.id(), Self::CHUNK_ID);
        debug_assert_eq!(chunk.len(), 4);
        debug_assert_eq!(chunk.pos(), 0);

        let fact = Self::read_inner(chunk)?;
        Ok(fact)
    }

    fn write_inner(self, writer: &mut impl Write) -> io::Result<()> {
        let Self { dw_sample_length } = self;

        writer.write_all(&dw_sample_length.to_le_bytes())
    }

    pub fn write_riff_chunk<W: Write + Seek>(self, writer: W) -> io::Result<()> {
        let mut chunk = RiffChunk::write_new(writer, Self::CHUNK_ID)?;

        self.write_inner(&mut chunk)?;
        chunk.update_header()?;

        debug_assert_eq!(chunk.len(), 4);
        debug_assert_eq!(chunk.pos(), chunk.len());

        Ok(())
    }
}

#[inline]
fn read_u16(reader: &mut impl Read) -> io::Result<u16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;

    Ok(u16::from_le_bytes(bytes))
}

#[inline]
fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;

    Ok(u32::from_le_bytes(bytes))
}
