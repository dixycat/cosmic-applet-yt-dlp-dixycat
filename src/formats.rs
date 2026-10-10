// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Display;

// Re-export the upstream types used by `From` impls so that any path
// adjustments only need to happen here if the crate reshuffles modules.
use yt_dlp::model::selector::{
    AudioCodecPreference as YtAudioCodec, AudioQuality as YtAudioQuality,
    VideoCodecPreference as YtVideoCodec, VideoQuality as YtVideoQuality,
};

// ---------------------------------------------------------------------------
// Video container format (MP4, MKV, WebM)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum VideoContainer {
    #[default]
    MP4,
    MKV,
    WebM,
}

impl VideoContainer {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoContainer::MP4 => "mp4",
            VideoContainer::MKV => "mkv",
            VideoContainer::WebM => "webm",
        }
    }
}

impl Display for VideoContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoContainer::MP4 => write!(f, "MP4"),
            VideoContainer::MKV => write!(f, "MKV"),
            VideoContainer::WebM => write!(f, "WebM"),
        }
    }
}

// ---------------------------------------------------------------------------
// Video quality (UI-facing labels)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum VideoQuality {
    #[default]
    Highest,
    FHD,
    HD,
    SD,
    Lowest,
}

impl Display for VideoQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoQuality::Highest => write!(f, "Highest"),
            VideoQuality::FHD => write!(f, "1080p"),
            VideoQuality::HD => write!(f, "720p"),
            VideoQuality::SD => write!(f, "480p"),
            VideoQuality::Lowest => write!(f, "Lowest"),
        }
    }
}

impl From<VideoQuality> for YtVideoQuality {
    fn from(val: VideoQuality) -> Self {
        match val {
            VideoQuality::Highest => YtVideoQuality::Best,
            VideoQuality::FHD => YtVideoQuality::High,
            VideoQuality::HD => YtVideoQuality::Medium,
            VideoQuality::SD => YtVideoQuality::Low,
            VideoQuality::Lowest => YtVideoQuality::Worst,
        }
    }
}

// ---------------------------------------------------------------------------
// Video codec (UI-facing labels)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum VideoCodec {
    AV1,
    AVC1,
    VP9,
    #[default]
    Any,
}

impl Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoCodec::AV1 => write!(f, "AV1"),
            VideoCodec::AVC1 => write!(f, "AVC1"),
            VideoCodec::VP9 => write!(f, "VP9"),
            VideoCodec::Any => write!(f, "Any"),
        }
    }
}

impl From<VideoCodec> for YtVideoCodec {
    fn from(val: VideoCodec) -> Self {
        match val {
            VideoCodec::AV1 => YtVideoCodec::AV1,
            VideoCodec::AVC1 => YtVideoCodec::AVC1,
            VideoCodec::VP9 => YtVideoCodec::VP9,
            VideoCodec::Any => YtVideoCodec::Any,
        }
    }
}

// ---------------------------------------------------------------------------
// Audio quality (UI-facing labels)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum AudioQuality {
    #[default]
    Best,
    High,
    Medium,
    Low,
    Worst,
}

impl Display for AudioQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioQuality::Best => write!(f, "Highest"),
            AudioQuality::High => write!(f, "High"),
            AudioQuality::Medium => write!(f, "Medium"),
            AudioQuality::Low => write!(f, "Low"),
            AudioQuality::Worst => write!(f, "Lowest"),
        }
    }
}

impl AudioQuality {
    pub fn ffmpeg_quality(&self, economy_mode: bool) -> &'static str {
        if economy_mode {
            match self {
                AudioQuality::Best | AudioQuality::High => "5",
                AudioQuality::Medium => "7",
                AudioQuality::Low | AudioQuality::Worst => "9",
            }
        } else {
            match self {
                AudioQuality::Best => "0",
                AudioQuality::High => "2",
                AudioQuality::Medium => "5",
                AudioQuality::Low => "7",
                AudioQuality::Worst => "9",
            }
        }
    }
}

impl From<AudioQuality> for YtAudioQuality {
    fn from(val: AudioQuality) -> Self {
        match val {
            AudioQuality::Best => YtAudioQuality::Best,
            AudioQuality::High => YtAudioQuality::High,
            AudioQuality::Medium => YtAudioQuality::Medium,
            AudioQuality::Low => YtAudioQuality::Low,
            AudioQuality::Worst => YtAudioQuality::Worst,
        }
    }
}

// ---------------------------------------------------------------------------
// Audio codec (UI-facing labels)
// ---------------------------------------------------------------------------

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum AudioCodec {
    MP3,
    AAC,
    Opus,
    FLAC,
    WAV,
    #[default]
    Any,
}

impl AudioCodec {
    /// Returns the yt-dlp audio format, preserving the source format for `Any`.
    pub fn yt_dlp_format(&self) -> &'static str {
        match self {
            AudioCodec::MP3 => "mp3",
            AudioCodec::AAC => "m4a",
            AudioCodec::Opus => "opus",
            AudioCodec::FLAC => "flac",
            AudioCodec::WAV => "wav",
            AudioCodec::Any => "best",
        }
    }

    pub fn converts_audio(&self) -> bool {
        !matches!(self, AudioCodec::Any)
    }

    pub fn supports_embedded_thumbnail(&self) -> bool {
        !matches!(self, AudioCodec::WAV)
    }
}

impl Display for AudioCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioCodec::MP3 => write!(f, "MP3"),
            AudioCodec::AAC => write!(f, "AAC (M4A)"),
            AudioCodec::Opus => write!(f, "Opus"),
            AudioCodec::FLAC => write!(f, "FLAC"),
            AudioCodec::WAV => write!(f, "WAV"),
            AudioCodec::Any => write!(f, "Original"),
        }
    }
}

impl From<AudioCodec> for YtAudioCodec {
    fn from(val: AudioCodec) -> Self {
        match val {
            AudioCodec::MP3 => YtAudioCodec::MP3,
            AudioCodec::AAC => YtAudioCodec::AAC,
            AudioCodec::Opus => YtAudioCodec::Opus,
            AudioCodec::FLAC => YtAudioCodec::Any,
            AudioCodec::WAV => YtAudioCodec::Any,
            AudioCodec::Any => YtAudioCodec::Any,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioCodec, AudioQuality};

    #[test]
    fn any_audio_codec_preserves_the_best_available_source_format() {
        assert_eq!(AudioCodec::Any.yt_dlp_format(), "best");
        assert_eq!(AudioCodec::Any.to_string(), "Original");
        assert!(!AudioCodec::Any.converts_audio());
        assert!(AudioCodec::MP3.converts_audio());
    }

    #[test]
    fn audio_quality_labels_do_not_claim_fixed_bitrates() {
        assert_eq!(AudioQuality::Best.to_string(), "Highest");
        assert_eq!(AudioQuality::High.to_string(), "High");
        assert_eq!(AudioQuality::Medium.to_string(), "Medium");
        assert_eq!(AudioQuality::Low.to_string(), "Low");
        assert_eq!(AudioQuality::Worst.to_string(), "Lowest");
    }

    #[test]
    fn audio_quality_maps_to_ffmpeg_quality_values() {
        assert_eq!(AudioQuality::Best.ffmpeg_quality(false), "0");
        assert_eq!(AudioQuality::High.ffmpeg_quality(false), "2");
        assert_eq!(AudioQuality::Medium.ffmpeg_quality(false), "5");
        assert_eq!(AudioQuality::Low.ffmpeg_quality(false), "7");
        assert_eq!(AudioQuality::Worst.ffmpeg_quality(false), "9");
        assert_eq!(AudioQuality::Best.ffmpeg_quality(true), "5");
    }

    #[test]
    fn thumbnail_embedding_support_covers_audio_codecs_that_support_covers() {
        for codec in [
            AudioCodec::MP3,
            AudioCodec::AAC,
            AudioCodec::Opus,
            AudioCodec::FLAC,
            AudioCodec::Any,
        ] {
            assert!(
                codec.supports_embedded_thumbnail(),
                "{codec} should support an embedded cover"
            );
        }
        assert!(!AudioCodec::WAV.supports_embedded_thumbnail());
    }
}
