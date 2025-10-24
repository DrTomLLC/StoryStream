// FILE: crates/media-engine/src/decoder.rs

use crate::error::{EngineError, EngineResult};
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, Signal, SignalSpec};
pub(crate) use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct AudioDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    spec: SignalSpec,
}

pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub spec: SignalSpec,
}

impl AudioDecoder {
    pub fn new(path: &Path) -> EngineResult<Self> {
        let file = std::fs::File::open(path)
            .map_err(|e| EngineError::DecodeError(format!("Failed to open file: {}", e)))?;

        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(extension);
        }

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| EngineError::DecodeError(format!("Failed to probe format: {}", e)))?;

        let reader = probed.format;

        let track = reader
            .default_track()
            .ok_or_else(|| EngineError::DecodeError("No audio track found".to_string()))?;

        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        let decoder = symphonia::default::get_codecs()
            .make(&codec_params, &DecoderOptions::default())
            .map_err(|e| EngineError::DecodeError(format!("Failed to create decoder: {}", e)))?;

        let spec = SignalSpec::new(
            codec_params.sample_rate.unwrap_or(44100),
            codec_params.channels.unwrap_or_default(),
        );

        Ok(Self {
            reader,
            decoder,
            track_id,
            spec,
        })
    }

    pub fn decode_next(&mut self) -> EngineResult<Option<DecodedAudio>> {
        loop {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(None);
                }
                Err(e) => {
                    return Err(EngineError::DecodeError(format!(
                        "Failed to read packet: {}",
                        e
                    )));
                }
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(e)) => {
                    log::warn!("Decode error, skipping packet: {}", e);
                    continue;
                }
                Err(e) => {
                    return Err(EngineError::DecodeError(format!(
                        "Failed to decode packet: {}",
                        e
                    )));
                }
            };

            let samples = convert_to_f32(&decoded)?;
            let spec = *decoded.spec();

            return Ok(Some(DecodedAudio { samples, spec }));
        }
    }

    pub fn spec(&self) -> &SignalSpec {
        &self.spec
    }

    pub fn seek(&mut self, time_secs: f64) -> EngineResult<()> {
        let sample_rate = self.spec.rate;
        let timestamp = (time_secs * sample_rate as f64) as u64;

        self.reader
            .seek(
                SeekMode::Accurate,
                SeekTo::TimeStamp {
                    ts: timestamp,
                    track_id: self.track_id,
                },
            )
            .map_err(|e| EngineError::SeekError(format!("Failed to seek: {}", e)))?;

        self.decoder.reset();

        Ok(())
    }
}

fn convert_to_f32(decoded: &AudioBufferRef) -> EngineResult<Vec<f32>> {
    match decoded {
        AudioBufferRef::F32(buf) => {
            let mut samples = Vec::with_capacity(buf.frames() * buf.spec().channels.count());
            for plane in buf.planes().planes() {
                samples.extend_from_slice(plane);
            }
            Ok(samples)
        }
        AudioBufferRef::S16(buf) => {
            let mut sample_buf = SampleBuffer::<f32>::new(buf.capacity() as u64, *buf.spec());
            sample_buf.copy_interleaved_ref(AudioBufferRef::S16(buf.clone()));
            Ok(sample_buf.samples().to_vec())
        }
        AudioBufferRef::S32(buf) => {
            let mut sample_buf = SampleBuffer::<f32>::new(buf.capacity() as u64, *buf.spec());
            sample_buf.copy_interleaved_ref(AudioBufferRef::S32(buf.clone()));
            Ok(sample_buf.samples().to_vec())
        }
        AudioBufferRef::U8(buf) => {
            let mut sample_buf = SampleBuffer::<f32>::new(buf.capacity() as u64, *buf.spec());
            sample_buf.copy_interleaved_ref(AudioBufferRef::U8(buf.clone()));
            Ok(sample_buf.samples().to_vec())
        }
        AudioBufferRef::U16(buf) => {
            let mut sample_buf = SampleBuffer::<f32>::new(buf.capacity() as u64, *buf.spec());
            sample_buf.copy_interleaved_ref(AudioBufferRef::U16(buf.clone()));
            Ok(sample_buf.samples().to_vec())
        }
        AudioBufferRef::U32(buf) => {
            let mut sample_buf = SampleBuffer::<f32>::new(buf.capacity() as u64, *buf.spec());
            sample_buf.copy_interleaved_ref(AudioBufferRef::U32(buf.clone()));
            Ok(sample_buf.samples().to_vec())
        }
        _ => Err(EngineError::DecodeError(
            "Unsupported sample format".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_nonexistent_file() {
        let result = AudioDecoder::new(Path::new("nonexistent.mp3"));
        assert!(result.is_err());
    }
    #[test]
    fn test_convert_formats() {
        // Just test that the function exists and compiles
        // Real testing would require actual audio data
    }
}
