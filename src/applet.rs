// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;
use std::path::PathBuf;

use cosmic::app::{Core, Task};
use cosmic::applet::padded_control;
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::destroy_popup;
use cosmic::iced::widget::{column, row};
use cosmic::iced::{window, Alignment, Length, Limits};
use cosmic::widget::segmented_button::{Entity, SingleSelectModel};
use cosmic::widget::text::body;
use cosmic::widget::{divider, segmented_control, text_input};
use cosmic::{Action, Application, Apply, Element};

use ashpd::desktop::file_chooser::SelectedFiles;
use notify_rust::Notification;

use crate::formats::{AudioCodec, AudioQuality, VideoCodec, VideoContainer, VideoQuality};
use crate::{fetcher, fl, fl_str};

use reqwest;
use serde_json;

// ---------------------------------------------------------------------------
// Debug logging support
// ---------------------------------------------------------------------------

static DEBUG_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn enable_debug() {
    DEBUG_MODE.store(true, std::sync::atomic::Ordering::Relaxed);
    eprintln!("[DEBUG] Verbose debug logging enabled");
}

pub fn is_debug() -> bool {
    DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn log_to_file(msg: &str) {
    if !is_debug() {
        return;
    }
    if let Ok(cache_dir) = xdg::BaseDirectories::with_prefix("cosmic-applet-yt-dlp-dixycat") {
        if let Ok(path) = cache_dir.place_cache_file("debug.log") {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(f, "{}", msg);
            }
        }
    }
}

macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::applet::is_debug() {
            let msg = format!("[DEBUG] {}", format_args!($($arg)*));
            eprintln!("{}", msg);
            $crate::applet::log_to_file(&msg);
        }
    };
}

const PACKAGED_DENO_PATH: &str = "/usr/lib/cosmic-applet-yt-dlp-dixycat/deno";

/// Locate the Deno runtime bundled with release packages. A system or
/// development runtime can be selected explicitly with COSMIC_YTDLP_DENO.
fn deno_runtime_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("COSMIC_YTDLP_DENO").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }

    let mut candidates = vec![PathBuf::from(PACKAGED_DENO_PATH)];
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join(".local/lib/cosmic-applet-yt-dlp-dixycat/deno"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("deno"));
        }
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("res/deno"));
    candidates.into_iter().find(|path| path.is_file())
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Parses yt-dlp CLI progress output line into (percent, speed_mbps, eta_secs, downloaded_bytes, total_bytes)
fn parse_ytdlp_progress_line(line: &str) -> Option<(f32, f64, Option<u64>, u64, u64)> {
    if !line.starts_with("[download]") {
        return None;
    }
    let rest = line.strip_prefix("[download]")?.trim();
    
    // Handle various progress line formats
    // Format 1: "45.2% of 12.34MiB at 2.45MiB/s ETA 00:03"
    // Format 2: "Downloading ~ 45.2% of 12.34MiB at 2.45MiB/s ETA 00:03"
    // Format 3: "45.2% at 2.45MiB/s ETA 00:03" (no total size yet)
    // Format 4: "Already downloaded" (skip)
    
    if rest.contains("Already downloaded") || rest.contains("has already been downloaded") {
        return Some((100.0, 0.0, Some(0), 0, 0));
    }
    
    // Find percentage - handle both "XX.X%" and "~ XX.X%"
    let pct_start = rest.find(|c: char| c.is_ascii_digit())?;
    let pct_end = rest[pct_start..].find('%')? + pct_start;
    let percent = rest[pct_start..pct_end].trim().parse::<f32>().ok()?;
    
    // Validate percentage is in reasonable range (avoid glitches)
    if percent < 0.0 || percent > 100.0 {
        return None;
    }

    let mut speed_mbps = 0.0;
    if let Some(at_idx) = rest.find(" at ") {
        let after_at = &rest[at_idx + 4..];
        let speed_token = after_at.split_whitespace().next().unwrap_or("");
        if speed_token.ends_with("MiB/s") || speed_token.ends_with("MB/s") {
            let num_str = speed_token.trim_end_matches("MiB/s").trim_end_matches("MB/s");
            speed_mbps = num_str.parse::<f64>().unwrap_or(0.0);
        } else if speed_token.ends_with("KiB/s") || speed_token.ends_with("KB/s") {
            let num_str = speed_token.trim_end_matches("KiB/s").trim_end_matches("KB/s");
            speed_mbps = num_str.parse::<f64>().unwrap_or(0.0) / 1024.0;
        } else if speed_token.ends_with("GiB/s") || speed_token.ends_with("GB/s") {
            let num_str = speed_token.trim_end_matches("GiB/s").trim_end_matches("GB/s");
            speed_mbps = num_str.parse::<f64>().unwrap_or(0.0) * 1024.0;
        }
    }

    let mut eta_secs = None;
    if let Some(eta_idx) = rest.find("ETA ") {
        let eta_token = rest[eta_idx + 4..].split_whitespace().next().unwrap_or("");
        let parts: Vec<&str> = eta_token.split(':').collect();
        if parts.len() == 2 {
            let mins = parts[0].parse::<u64>().unwrap_or(0);
            let secs = parts[1].parse::<u64>().unwrap_or(0);
            eta_secs = Some(mins * 60 + secs);
        } else if parts.len() == 3 {
            let hours = parts[0].parse::<u64>().unwrap_or(0);
            let mins = parts[1].parse::<u64>().unwrap_or(0);
            let secs = parts[2].parse::<u64>().unwrap_or(0);
            eta_secs = Some(hours * 3600 + mins * 60 + secs);
        }
    }

    let mut total_bytes = 0u64;
    if let Some(of_idx) = rest.find(" of ") {
        let after_of = &rest[of_idx + 4..];
        let total_token = after_of.trim_start_matches('~').trim().split_whitespace().next().unwrap_or("");
        if total_token.ends_with("MiB") || total_token.ends_with("MB") {
            let num = total_token.trim_end_matches("MiB").trim_end_matches("MB").parse::<f64>().unwrap_or(0.0);
            total_bytes = (num * 1_048_576.0) as u64;
        } else if total_token.ends_with("KiB") || total_token.ends_with("KB") {
            let num = total_token.trim_end_matches("KiB").trim_end_matches("KB").parse::<f64>().unwrap_or(0.0);
            total_bytes = (num * 1024.0) as u64;
        } else if total_token.ends_with("GiB") || total_token.ends_with("GB") {
            let num = total_token.trim_end_matches("GiB").trim_end_matches("GB").parse::<f64>().unwrap_or(0.0);
            total_bytes = (num * 1_073_741_824.0) as u64;
        }
    }

    let downloaded_bytes = if total_bytes > 0 {
        ((percent / 100.0) * total_bytes as f32) as u64
    } else {
        0
    };

    Some((percent, speed_mbps, eta_secs, downloaded_bytes, total_bytes))
}

/// Cleans up empty (0-byte) files and leftover temporary files from `dir`.
async fn cleanup_zero_and_temp_files(dir: &std::path::Path) {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(meta) = entry.metadata().await else {
            continue;
        };
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if meta.is_file() {
            // Remove only files that yt-dlp/ffmpeg explicitly use as temporary
            // artifacts; unrelated empty files in the download directory are safe.
            let is_temp = name.ends_with(".part")
                || name.ends_with(".ytdl")
                || name.ends_with(".temp")
                || name.contains(".part-")
                || name.contains(".temp.")
                || name.starts_with("temp_video_")
                || name.starts_with("temp_audio_")
                || (meta.len() == 0
                    && (name.ends_with(".mp4")
                        || name.ends_with(".mkv")
                        || name.ends_with(".webm")
                        || name.ends_with(".part")));
            if is_temp {
                debug_log!("Removing leftover temporary file: {:?}", entry.path());
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }
    }
}

async fn cleanup_subtitle_sidecars(dir: &std::path::Path, title: &str) {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };
    let title = title.trim();
    if title.is_empty() {
        return;
    }
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("vtt") {
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let belongs_to_download = stem == title
                || stem
                    .strip_prefix(title)
                    .is_some_and(|suffix| suffix.starts_with('.'));
            if belongs_to_download {
                debug_log!("Removing embedded-only subtitle sidecar: {:?}", path);
                let _ = tokio::fs::remove_file(path).await;
            }
        }
    }
}

async fn subtitle_rate_limit_detected(
    stderr_log: &std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
) -> bool {
    let lines = stderr_log.lock().await;
    lines.iter().any(|line| {
        line.contains("subtitle") && (line.contains("429") || line.contains("Too Many Requests"))
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DownloadStage {
    #[default]
    Preparing,
    Downloading,
    Subtitles,
    PostProcessing,
}

/// Checks if python's `mutagen` package is available for embedding album cover art into Opus/FLAC.
async fn has_python_mutagen() -> bool {
    tokio::process::Command::new("python3")
        .args(["-c", "import mutagen"])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn is_youtube_url(url: &str) -> bool {
    let normalized = url.to_ascii_lowercase();
    normalized.contains("youtube.com")
        || normalized.contains("youtu.be")
        || normalized.contains("music.youtube.com")
}

fn is_tiktok_url(url: &str) -> bool {
    let normalized = url.to_ascii_lowercase();
    normalized.contains("tiktok.com")
}

fn is_spotify_url(url: &str) -> bool {
    let normalized = url.to_ascii_lowercase();
    normalized.contains("spotify.com")
        || normalized.contains("open.spotify.com")
}

fn unsupported_download_reason(url: &str) -> Option<&'static str> {
    if is_spotify_url(url) {
        Some("Spotify usa DRM e não pode ser baixado por este applet.")
    } else if is_tiktok_url(url) {
        Some("TikTok pode falhar por limitação do extractor do yt-dlp. Tente outro link ou atualize o yt-dlp.")
    } else {
        None
    }
}

fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let trimmed = version.trim().trim_start_matches('v');
    let core = trimmed.split('-').next().unwrap_or(trimmed);
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    Some((major, minor, patch))
}

fn is_version_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(candidate_v), Some(current_v)) => candidate_v > current_v,
        _ => candidate > current,
    }
}

/// Runs a download through the unified yt-dlp native binary engine with live progress streaming.
#[allow(clippy::too_many_arguments)]
async fn run_download_job(
    download_id: u32,
    video_selected: bool,
    url: &str,
    custom_name: Option<String>,
    video_container_ext: &str,
    downloader: &yt_dlp::Downloader,
    output_dir_ref: &std::path::Path,
    audio_ext: &str,
    video_quality: VideoQuality,
    audio_quality: AudioQuality,
    economy_mode: bool,
    subtitle_mode: SubtitleMode,
    cancel_rx: &mut tokio::sync::oneshot::Receiver<()>,
    output: &mut cosmic::iced::futures::channel::mpsc::Sender<cosmic::Action<Message>>,
    mut notify: notify_rust::Notification,
) {
    use cosmic::iced::futures::SinkExt;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt};

    debug_log!("Starting download job #{}: url={:?}, video_selected={}, format={}", download_id, url, video_selected, if video_selected { video_container_ext } else { audio_ext });
    let _ = output
        .send(cosmic::Action::App(Message::DownloadStage {
            id: download_id,
            stage: DownloadStage::Preparing,
        }))
        .await;

    if let Some(reason) = unsupported_download_reason(url) {
        let url_owned = url.to_owned();
        tokio::spawn(async move {
            let title = if is_spotify_url(&url_owned) {
                "Spotify protegido por DRM"
            } else if is_tiktok_url(&url_owned) {
                "TikTok não suportado pelo extractor"
            } else {
                "Download indisponível"
            };
            let _ = notify.summary(title).body(reason).show_async().await;
        });
        let _ = output.send(cosmic::Action::App(Message::Finished(download_id))).await;
        return;
    }

    // Detect if this is a playlist URL
    let is_playlist = url.contains("list=") || url.contains("/playlist");

    // Extract the title for the notification. This is its own yt-dlp process,
    // so it must also observe cancellation before the actual download starts.
    let mut title_cmd = tokio::process::Command::new(&downloader.libraries().youtube);
    title_cmd
        .kill_on_drop(true)
        .arg("--force-ipv4")
        .arg("--print").arg("%(title)s")
        .arg("--no-warnings")
        .arg("--no-playlist")
        .stdout(std::process::Stdio::piped());
    if is_youtube_url(url) {
        title_cmd
            .arg("--extractor-args")
            .arg("youtube:player_client=default,web_music,mweb,ios");
    }
    if let Some(deno_path) = deno_runtime_path() {
        title_cmd
            .arg("--js-runtimes")
            .arg(format!("deno:{}", deno_path.display()));
    }
    title_cmd.arg(url);
    let mut title_bytes = Vec::new();
    let title_output = match title_cmd.spawn() {
        Ok(mut title_child) => {
            if let Some(mut title_stdout) = title_child.stdout.take() {
                tokio::select! {
                    _ = &mut *cancel_rx => {
                        debug_log!("Download #{} was cancelled while resolving its title", download_id);
                        let _ = title_child.start_kill();
                        let _ = title_child.wait().await;
                        cleanup_zero_and_temp_files(output_dir_ref).await;
                        let _ = output.send(cosmic::Action::App(Message::Finished(download_id))).await;
                        return;
                    }
                    result = async {
                        let _ = title_stdout.read_to_end(&mut title_bytes).await;
                        title_child.wait().await
                    } => result.ok(),
                }
            } else {
                None
            }
        }
        Err(error) => {
            debug_log!("Could not start title lookup for download #{}: {:?}", download_id, error);
            None
        }
    };

    let extracted_title = title_output
        .and_then(|_| String::from_utf8(title_bytes).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let display_title = custom_name.clone()
        .or(extracted_title)
        .unwrap_or_else(|| "Download".to_string());

    let out_template = if let Some(ref name) = custom_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            format!("{}/{}.%(ext)s", output_dir_ref.display(), trimmed)
        } else {
            format!("{}/%(title)s.%(ext)s", output_dir_ref.display())
        }
    } else {
        format!("{}/%(title)s.%(ext)s", output_dir_ref.display())
    };

    let ffmpeg_path = &downloader.libraries().ffmpeg;
    let ffmpeg_dir = ffmpeg_path.parent().unwrap_or_else(|| std::path::Path::new("/usr/bin"));
    let has_ffprobe = ffmpeg_dir.join("ffprobe").exists()
        || std::path::Path::new("/usr/bin/ffprobe").exists();

    let mut cmd = tokio::process::Command::new(&downloader.libraries().youtube);
    cmd.kill_on_drop(true);
    if is_debug() {
        cmd.arg("--verbose");
    }
    cmd.arg("--ffmpeg-location").arg(ffmpeg_dir);
    cmd.arg("--force-ipv4");
    if let Some(deno_path) = deno_runtime_path() {
        debug_log!("Using bundled Deno runtime: {:?}", deno_path);
        cmd.arg("--js-runtimes")
            .arg(format!("deno:{}", deno_path.display()));
    } else {
        debug_log!("Bundled Deno runtime was not found");
    }
    if is_youtube_url(url) {
        // `web_music` makes Music URLs use their dedicated Innertube client;
        // the remaining clients provide fallbacks when one is rate-limited.
        cmd.arg("--extractor-args").arg("youtube:player_client=default,web_music,mweb,ios");
    }
    cmd.arg("--socket-timeout").arg("30");
    cmd.arg("--retries").arg("10");
    cmd.arg("--fragment-retries").arg("10");
    cmd.arg("--retry-sleep").arg("2");
    cmd.arg("--newline");
    cmd.arg("--progress");
    cmd.arg("--no-mtime"); // Set modification time to download time
    cmd.arg("-o").arg(&out_template);

    if is_playlist {
        cmd.arg("--yes-playlist");
    } else {
        cmd.arg("--no-playlist");
    }

    if video_selected {
        // Video format selection
        let format_filter = if economy_mode {
            // Economy mode: low resolution to save data
            match video_quality {
                VideoQuality::Highest | VideoQuality::FHD | VideoQuality::HD => "bestvideo[height<=360]+bestaudio/best[height<=360]/best",
                VideoQuality::SD => "bestvideo[height<=240]+bestaudio/best[height<=240]/best",
                VideoQuality::Lowest => "worstvideo+worstaudio/worst",
            }
        } else {
            match video_quality {
                VideoQuality::Highest => "bestvideo+bestaudio/best",
                VideoQuality::FHD => "bestvideo[height<=1080]+bestaudio/best[height<=1080]/best",
                VideoQuality::HD => "bestvideo[height<=720]+bestaudio/best[height<=720]/best",
                VideoQuality::SD => "bestvideo[height<=480]+bestaudio/best[height<=480]/best",
                VideoQuality::Lowest => "worstvideo+worstaudio/worst",
            }
        };
        cmd.arg("-f").arg(format_filter);

        if video_container_ext == "mp4" {
            cmd.arg("--merge-output-format").arg("mp4");
            cmd.arg("--remux-video").arg("mp4");
        } else if video_container_ext == "mkv" {
            cmd.arg("--merge-output-format").arg("mkv");
            cmd.arg("--remux-video").arg("mkv");
        } else if video_container_ext == "webm" {
            cmd.arg("--merge-output-format").arg("webm");
            cmd.arg("--remux-video").arg("webm");
        }
        // Embed title, artist (uploader) and thumbnail cover into the video file
        cmd.arg("--embed-metadata");
        if has_ffprobe && !economy_mode {
            cmd.arg("--embed-thumbnail");
        }
        // Subtitle flags based on user selection (only for video and YouTube).
        // Non-YouTube extractors do not reliably expose YouTube subtitle metadata,
        // and applying the YouTube-only flags there can trigger extractor errors.
        if subtitle_mode != SubtitleMode::Off && is_youtube_url(url) {
            let sub_fmt = if video_container_ext == "mkv" {
                "ass/srt/best"
            } else {
                "vtt/best"
            };

            cmd.arg("--sub-langs").arg("pt-BR,pt,en-US");
            cmd.arg("--ignore-errors");
            cmd.arg("--sleep-subtitles").arg("2");

            // Both modes download the subtitle so ffmpeg can embed it.
            cmd.arg("--embed-subs");
            cmd.arg("--sub-format").arg(sub_fmt);
            cmd.arg("--write-sub");
            cmd.arg("--write-auto-sub");
        }
        cmd.arg("--add-metadata");
        cmd.arg("--parse-metadata").arg("%(uploader)s:%(artist)s");
    } else {
        // Audio extraction
        cmd.arg("-x");
        cmd.arg("--audio-format").arg(audio_ext);
        let audio_q = if economy_mode {
            // Economy mode: lower bitrate to save data
            match audio_quality {
                AudioQuality::Best | AudioQuality::High => "5",
                AudioQuality::Medium => "7",
                AudioQuality::Low | AudioQuality::Worst => "9",
            }
        } else {
            match audio_quality {
                AudioQuality::Best => "0",
                AudioQuality::High => "2",
                AudioQuality::Medium => "5",
                AudioQuality::Low => "7",
                AudioQuality::Worst => "9",
            }
        };
        cmd.arg("--audio-quality").arg(audio_q);
        // Embed title, artist, album art (thumbnail) into audio file
        cmd.arg("--embed-metadata");
        // For audio, ffmpeg natively embeds thumbnails into MP3 and M4A/AAC without extra dependencies.
        // Formats like Opus and FLAC require python's 'mutagen' module. Without mutagen, yt-dlp fails
        // with exit code 1 and leaves orphaned .webp/.png thumbnails behind.
        // WAV does not support embedded thumbnails in yt-dlp.
        let can_embed_audio_thumb = has_ffprobe && !economy_mode && match audio_ext {
            "mp3" | "m4a" | "aac" => true,
            "opus" | "flac" => has_python_mutagen().await,
            _ => false,
        };
        if can_embed_audio_thumb {
            cmd.arg("--embed-thumbnail");
        }
        cmd.arg("--add-metadata");
        cmd.arg("--embed-chapters");
        // Map uploader -> artist, channel -> album_artist for music players
        cmd.arg("--parse-metadata").arg("%(uploader)s:%(artist)s");
        cmd.arg("--parse-metadata").arg("%(channel)s:%(album_artist)s");
    }

    cmd.arg(url);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    debug_log!("Spawning yt-dlp command: {:?}", cmd);
    let _ = output
        .send(cosmic::Action::App(Message::DownloadStage {
            id: download_id,
            stage: DownloadStage::Downloading,
        }))
        .await;

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            debug_log!("Failed to spawn yt-dlp: {:?}", e);
            tokio::spawn(async move {
                let _ = notify.summary(&fl_str!("download-failed", title = display_title)).show_async().await;
            });
            let _ = output.send(cosmic::Action::App(Message::Finished(download_id))).await;
            return;
        }
    };

    let stderr_log = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));

    // Continuously drain stderr to avoid Linux pipe buffer deadlock (64KB)
    if let Some(stderr) = child.stderr.take() {
        let mut reader = tokio::io::BufReader::new(stderr).lines();
        let stderr_log_clone = stderr_log.clone();
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                debug_log!("[yt-dlp stderr #{}] {}", download_id, line);
                let mut guard = stderr_log_clone.lock().await;
                guard.push(line);
                let len = guard.len();
                if len > 25 {
                    let retain_from = len - 25;
                    guard.drain(..retain_from);
                }
            }
        });
    }

    if let Some(stdout) = child.stdout.take() {
        let mut reader = tokio::io::BufReader::new(stdout).lines();
        let mut progress_out = output.clone();

        tokio::spawn(async move {
            use cosmic::iced::futures::SinkExt as _;
            while let Ok(Some(line)) = reader.next_line().await {
                debug_log!("[yt-dlp stdout #{}] {}", download_id, line);
                if line.contains("Downloading subtitles") || line.contains("Writing video subtitles") {
                    let _ = progress_out
                        .send(cosmic::Action::App(Message::DownloadStage {
                            id: download_id,
                            stage: DownloadStage::Subtitles,
                        }))
                        .await;
                } else if line.contains("[Merger]") || line.contains("[ExtractAudio]") || line.contains("[Fixup]") || line.contains("[ffmpeg]") || line.contains("[VideoRemuxer]") {
                    let _ = progress_out.send(cosmic::Action::App(Message::DownloadProgress {
                        id: download_id,
                        percent: 100.0,
                        speed_mbps: 0.0,
                        eta_secs: None,
                        downloaded_bytes: 0,
                        total_bytes: 0,
                        is_post_processing: true,
                    })).await;
                    let _ = progress_out
                        .send(cosmic::Action::App(Message::DownloadStage {
                            id: download_id,
                            stage: DownloadStage::PostProcessing,
                        }))
                        .await;
                } else if line.contains("[download] Downloading item") || line.contains("[download] Downloading video") {
                    // Playlist item progress
                    if let Some(item_idx) = line.find("item ") {
                        let after = &line[item_idx + 5..];
                        if let Some(of_idx) = after.find(" of ") {
                            let cur = after[..of_idx].trim().parse::<u32>().unwrap_or(1);
                            let tot = after[of_idx + 4..].split_whitespace().next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
                            let _ = progress_out.send(cosmic::Action::App(Message::PlaylistProgress {
                                id: download_id,
                                current: cur,
                                total: tot,
                                video_title: String::new(),
                            })).await;
                        }
                    }
                } else if let Some((percent, speed_mbps, eta_secs, dl_bytes, tot_bytes)) = parse_ytdlp_progress_line(&line) {
                    let _ = progress_out.send(cosmic::Action::App(Message::DownloadProgress {
                        id: download_id,
                        percent,
                        speed_mbps,
                        eta_secs,
                        downloaded_bytes: dl_bytes,
                        total_bytes: tot_bytes,
                        is_post_processing: false,
                    })).await;
                }
            }
        });
    }

    let status = tokio::select! {
        _ = &mut *cancel_rx => {
            debug_log!("Download #{} was cancelled by user. Terminating process...", download_id);
            let _ = child.start_kill();
            let _ = child.wait().await;
            cleanup_zero_and_temp_files(output_dir_ref).await;
            let _ = output.send(cosmic::Action::App(Message::Finished(download_id))).await;
            return;
        }
        res = child.wait() => res,
    };
    debug_log!("Download #{} exited with status: {:?}", download_id, status);
    cleanup_zero_and_temp_files(output_dir_ref).await;

    if video_selected && subtitle_mode == SubtitleMode::Embedded {
        cleanup_subtitle_sidecars(output_dir_ref, &display_title).await;
    }

    // Safety cleanup: If an audio download occurred and orphaned thumbnail files (.webp / .png)
    // were left behind, remove them to keep the user's music folder clean.
    if !video_selected {
        if let Ok(mut entries) = tokio::fs::read_dir(output_dir_ref).await {
            let mut audio_stems = std::collections::HashSet::new();
            let mut img_candidates = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                let p = entry.path();
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ext == audio_ext {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            audio_stems.insert(stem.to_string());
                        }
                    } else if ext == "webp" || ext == "png" {
                        img_candidates.push(p);
                    }
                }
            }
            for img_path in img_candidates {
                if let Some(stem) = img_path.file_stem().and_then(|s| s.to_str()) {
                    if audio_stems.contains(stem) {
                        let _ = tokio::fs::remove_file(img_path).await;
                    }
                }
            }
        }
    }

    // Check if download failed and attempt auto-update of yt-dlp
    if !status.as_ref().map_or(false, |s| s.success()) {
        let _ffmpeg_path = &downloader.libraries().ffmpeg;
        let youtube_path = &downloader.libraries().youtube;
        
        // Attempt auto-update of yt-dlp when download fails
        let update_result = tokio::process::Command::new(youtube_path)
            .args(["--update-to", "stable"])
            .output()
            .await;
        
        if let Ok(update_out) = update_result {
            if update_out.status.success() {
                // Notify user that yt-dlp was updated and they should retry
                let mut notify_update = notify.clone();
                tokio::spawn(async move {
                    let _ = notify_update
                        .summary("yt-dlp atualizado")
                        .body("O yt-dlp foi atualizado. Tente baixar novamente.")
                        .show_async()
                        .await;
                });
            }
        }
    }

    if status.map_or(false, |s| s.success()) {
        let subtitle_warning = if subtitle_mode != SubtitleMode::Off
            && subtitle_rate_limit_detected(&stderr_log).await
        {
            Some(fl_str!("subtitle-partial").to_string())
        } else {
            None
        };
        tokio::spawn(async move {
            let mut notification = notify.summary(&fl_str!("finished-download", title = display_title));
            if let Some(body) = subtitle_warning {
                notification = notification.body(&body);
            }
            let _ = notification.show_async().await;
        });
    } else {
        let stderr_lines = stderr_log.lock().await;
        let joined = stderr_lines.join("\n");
        let friendly = if joined.contains("DRM") {
            Some("Este site usa DRM e não pode ser baixado por este applet.")
        } else if is_tiktok_url(url) {
            Some("TikTok pode falhar por limitação do extractor do yt-dlp. Tente outro link ou atualize o yt-dlp.")
        } else {
            None
        };
        drop(stderr_lines);

        let url_owned = url.to_owned();
        tokio::spawn(async move {
            if let Some(body) = friendly {
                let title = if is_spotify_url(&url_owned) {
                    "Spotify protegido por DRM"
                } else if is_tiktok_url(&url_owned) {
                    "TikTok não suportado pelo extractor"
                } else {
                    "Download indisponível"
                };
                let _ = notify.summary(title).body(body).show_async().await;
            } else {
                let _ = notify.summary(&fl_str!("download-failed", title = display_title)).show_async().await;
            }
        });
    }

    let _ = output.send(cosmic::Action::App(Message::Finished(download_id))).await;
}

// ---------------------------------------------------------------------------
// Static dropdown option lists
// ---------------------------------------------------------------------------

const VIDEO_CONTAINERS: &[VideoContainer] = &[
    VideoContainer::MP4,
    VideoContainer::MKV,
    VideoContainer::WebM,
];
const VIDEO_CONTAINER_LABELS: &[&str] = &["MP4", "MKV", "WebM"];

const VIDEO_QUALITIES: &[VideoQuality] = &[
    VideoQuality::Highest,
    VideoQuality::FHD,
    VideoQuality::HD,
    VideoQuality::SD,
    VideoQuality::Lowest,
];
const VIDEO_QUALITY_LABELS: &[&str] = &["Highest", "1080p", "720p", "480p", "Lowest"];

const VIDEO_CODECS: &[VideoCodec] = &[
    VideoCodec::AV1,
    VideoCodec::AVC1,
    VideoCodec::VP9,
    VideoCodec::Any,
];
const VIDEO_CODEC_LABELS: &[&str] = &["AV1", "AVC1", "VP9", "Any"];

const AUDIO_QUALITIES: &[AudioQuality] = &[
    AudioQuality::Best,
    AudioQuality::High,
    AudioQuality::Medium,
    AudioQuality::Low,
    AudioQuality::Worst,
];
const AUDIO_QUALITY_LABELS: &[&str] = &["Highest", "192kbps", "128kbps", "96kbps", "Lowest"];

const AUDIO_CODECS: &[AudioCodec] = &[
    AudioCodec::MP3,
    AudioCodec::AAC,
    AudioCodec::Opus,
    AudioCodec::FLAC,
    AudioCodec::WAV,
    AudioCodec::Any,
];
const AUDIO_CODEC_LABELS: &[&str] = &["MP3", "AAC (M4A)", "Opus", "FLAC", "WAV", "Any"];

// ---------------------------------------------------------------------------
// Subtitle mode
// ---------------------------------------------------------------------------

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum SubtitleMode {
    #[default]
    Off,
    Embedded,
    EmbeddedAndVtt,
}

const SUBTITLE_MODES: &[SubtitleMode] = &[
    SubtitleMode::Off,
    SubtitleMode::Embedded,
    SubtitleMode::EmbeddedAndVtt,
];

// ---------------------------------------------------------------------------
// Per-download progress state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ActiveDownload {
    pub id: u32,
    pub percent: f32,
    pub speed_mbps: f64,
    pub eta_secs: Option<u64>,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub is_post_processing: bool,
    pub is_audio: bool,
    pub stage: DownloadStage,
    // Playlist tracking (None for single-video downloads)
    pub playlist_current: Option<u32>,
    pub playlist_total: Option<u32>,
    pub playlist_title: Option<String>,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct Ytdlp {
    core: Core,

    download_type: SingleSelectModel,
    video_entity: Entity,

    video_folder: String,
    audio_folder: String,
    url: String,
    custom_name: String,

    video_container: VideoContainer,
    video_quality: VideoQuality,
    audio_quality: AudioQuality,
    video_codec: VideoCodec,
    audio_codec: AudioCodec,

    lib_dir: PathBuf,
    popup: Option<window::Id>,

    active_downloads: Vec<ActiveDownload>,
    next_download_id: u32,
    cancel_senders: HashMap<u32, tokio::sync::oneshot::Sender<()>>,
    show_platforms: bool,
    show_about: bool,
    
    // Economy mode for data saving
    economy_mode: bool,

    // Subtitle mode and labels
    subtitle_mode: SubtitleMode,
    subtitle_labels: [String; 3],

    // Update checking state
    update_available: Option<ReleaseInfo>,
    is_checking_updates: bool,
    is_installing_update: bool,
    // Last check result message shown in tooltip
    update_last_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(window::Id),
    EnterURL(String),
    EnterCustomName(String),
    SelectFolder,
    ProcessSelectFolder(String),
    ChangeType(Entity),
    VideoContainerSelected(usize),
    VideoQualitySelected(usize),
    AudioQualitySelected(usize),
    VideoCodecSelected(usize),
    AudioCodecSelected(usize),
    Download,
    CancelDownload(u32),
    DownloadProgress {
        id: u32,
        percent: f32,
        speed_mbps: f64,
        eta_secs: Option<u64>,
        downloaded_bytes: u64,
        total_bytes: u64,
        is_post_processing: bool,
    },
    DownloadStage {
        id: u32,
        stage: DownloadStage,
    },
    /// Update playlist per-item progress counter
    PlaylistProgress {
        id: u32,
        current: u32,
        total: u32,
        video_title: String,
    },
    Finished(u32),
    TogglePlatforms,
    ToggleAbout,
    /// Surface action forwarded from popup_dropdown
    SurfaceAction(cosmic::surface::Action<Message>),
    /// Toggle economy mode for data saving
    #[allow(dead_code)]
    ToggleEconomyMode,
    /// Subtitle mode dropdown selection
    SubtitleModeSelected(usize),
    /// Check for updates from GitHub releases
    CheckForUpdates,
    /// Update check result
    UpdateCheckResult(Result<Option<ReleaseInfo>, String>),
    /// Install update with polkit
    InstallUpdate,
    /// Installation result
    InstallResult(Result<(), String>),
    /// The updated binary could not replace the currently running applet.
    RestartFailed(String),
}

/// Release information from GitHub API
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    #[allow(dead_code)]
    pub version: String,
    pub tag_name: String,
    pub download_url: String,
    #[allow(dead_code)]
    pub release_notes: String,
}

impl Application for Ytdlp {
    type Executor = cosmic::executor::Default;
    type Flags = PathBuf;
    type Message = Message;

    const APP_ID: &'static str = "io.github.felix_the_cat177.CosmicAppletYtDlp";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut download_type = SingleSelectModel::default();
        let video_entity = download_type.insert().text(fl!("video")).id();
        download_type.insert().text(fl!("audio"));
        download_type.activate(video_entity);

        let video_folder = xdg_user::videos()
            .ok()
            .flatten()
            .map_or(String::from("~/Videos"), |path| {
                String::from(path.to_string_lossy())
            });
        let audio_folder = xdg_user::music()
            .ok()
            .flatten()
            .map_or(String::from("~/Music"), |path| {
                String::from(path.to_string_lossy())
            });

        let subtitle_labels = [
            fl!("subtitle-no"),
            fl!("subtitle-yes"),
            fl!("subtitle-yes-vtt"),
        ];

        let app = Ytdlp {
            core,
            download_type,
            video_entity,
            video_folder,
            audio_folder,
            lib_dir: flags,
            subtitle_labels,
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("multimedia-video-player-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Self::Message> {
        let video_selected = self.video_entity == self.download_type.active();
        let pad = self.core.applet.suggested_padding(true);
        let Spacing {
            space_xxs, space_s, ..
        } = cosmic::theme::active().cosmic().spacing;

        // Determine update button icon and tooltip based on current state
        let update_icon_name = if self.update_available.is_some() {
            "software-update-available-symbolic"
        } else if self.is_checking_updates {
            "emblem-synchronizing-symbolic"
        } else {
            "view-refresh-symbolic"
        };
        let update_tooltip_text = if self.is_checking_updates {
            fl!("checking-updates")
        } else {
            self.update_last_msg
                .clone()
                .unwrap_or_else(|| fl!("check-updates"))
        };
        let update_btn: Element<'_, Message> = if self.update_available.is_some() {
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name(update_icon_name)
            )
            .on_press(Message::InstallUpdate)
            .into()
        } else if self.is_checking_updates {
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name(update_icon_name)
            )
            .into()
        } else {
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name(update_icon_name)
            )
            .on_press(Message::CheckForUpdates)
            .into()
        };

        let mut content = column![
            // Information buttons above the URL field
            row![
                cosmic::widget::tooltip(
                    cosmic::widget::button::icon(
                        cosmic::widget::icon::from_name("help-about-symbolic")
                    )
                    .on_press(Message::ToggleAbout),
                    cosmic::widget::text::body(fl!("about-tooltip")),
                    cosmic::widget::tooltip::Position::Bottom,
                ),
                cosmic::widget::tooltip(
                    cosmic::widget::button::standard(fl!("platforms-button"))
                        .on_press(Message::TogglePlatforms),
                    cosmic::widget::text::body(fl!("platforms-tooltip")),
                    cosmic::widget::tooltip::Position::Bottom,
                ),
                cosmic::widget::tooltip(
                    update_btn,
                    cosmic::widget::text::body(update_tooltip_text),
                    cosmic::widget::tooltip::Position::Bottom,
                ),
            ]
            .align_y(Alignment::Center)
            .spacing(space_xxs)
            .apply(padded_control)
            .width(Length::Fill),
            text_input(fl!("url"), &self.url)
                .on_input(Message::EnterURL)
                .apply(padded_control)
                .width(Length::Fill),
            // Custom file name input (optional)
            text_input(fl!("filename"), &self.custom_name)
                .on_input(Message::EnterCustomName)
                .apply(padded_control)
                .width(Length::Fill),
            segmented_control::horizontal(&self.download_type)
                .on_activate(Message::ChangeType)
                .apply(padded_control)
                .width(Length::Fill),
            if video_selected {
                self.view_video(self.popup)
            } else {
                self.view_audio(self.popup)
            },
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
            row![
                body(fl!("folder")).width(Length::Fill),
                cosmic::widget::button::standard(fl!("browse"))
                    .on_press(Message::SelectFolder)
            ]
            .align_y(Alignment::Center)
            .spacing(pad.0)
            .apply(padded_control),
            text_input(
                "",
                if video_selected {
                    self.video_folder.clone()
                } else {
                    self.audio_folder.clone()
                }
            )
            .on_focus(Message::SelectFolder)
            .on_input(Message::ProcessSelectFolder)
            .apply(padded_control),
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
            {
                let active_count = self.active_downloads.len() as u32;
                row![
                    body(fl!("downloading", total = active_count))
                        .width(Length::Fill),
                    cosmic::widget::button::suggested(fl!("download"))
                        .on_press(Message::Download),
                ]
                .align_y(Alignment::Center)
                .spacing(pad.0)
                .apply(padded_control)
            },
        ]
        .padding([pad.0, pad.1]);

        // Show platforms panel if toggled
        if self.show_platforms {
            content = content.push(self.view_platforms());
        }

        if self.show_about {
            content = content.push(self.view_about());
        }

        // Append a progress row for each active download
        for dl in &self.active_downloads {
            content = content.push(self.view_progress(dl));
        }

        let scrollable_content = cosmic::widget::scrollable(content.width(Length::Fixed(480.0)))
            .height(Length::Shrink)
            .width(Length::Fill);

        self.core.applet.popup_container(scrollable_content).into()
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    cosmic::surface::surface_task(cosmic::surface::action::app_popup(
                        |_| Default::default(),
                        move |app: &mut Self| {
                            let new_id = window::Id::unique();
                            app.popup.replace(new_id);
                            let mut popup_settings = app.core.applet.get_popup_settings(
                                app.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );
                            popup_settings.positioner.size_limits = Limits::NONE
                                .max_width(800.0)
                                .min_width(440.0)
                                .min_height(200.0)
                                .max_height(850.0);
                            popup_settings
                        },
                        None,
                    ))
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::EnterURL(url) => self.url = url,
            Message::EnterCustomName(name) => self.custom_name = name,
            Message::ChangeType(id) => self.download_type.activate(id),
            Message::VideoContainerSelected(idx) => {
                if let Some(&c) = VIDEO_CONTAINERS.get(idx) {
                    self.video_container = c;
                }
            }
            Message::VideoQualitySelected(idx) => {
                if let Some(&q) = VIDEO_QUALITIES.get(idx) {
                    self.video_quality = q;
                }
            }
            Message::AudioQualitySelected(idx) => {
                if let Some(&q) = AUDIO_QUALITIES.get(idx) {
                    self.audio_quality = q;
                }
            }
            Message::VideoCodecSelected(idx) => {
                if let Some(&c) = VIDEO_CODECS.get(idx) {
                    self.video_codec = c;
                }
            }
            Message::AudioCodecSelected(idx) => {
                if let Some(&c) = AUDIO_CODECS.get(idx) {
                    self.audio_codec = c;
                }
            }
            Message::SelectFolder => {
                let future = async {
                    let request = SelectedFiles::open_file()
                        .title("Download Folder")
                        .accept_label("Select")
                        .directory(true)
                        .multiple(false)
                        .modal(true)
                        .send()
                        .await
                        .ok()?;
                    let folder = request.response().ok()?;
                    let uri = folder.uris().first()?;
                    uri.to_file_path().ok().map(|p| p.to_string_lossy().into_owned())
                };
                return Task::perform(future, |folder| {
                    if let Some(folder) = folder {
                        return Action::App(Message::ProcessSelectFolder(folder));
                    }
                    Action::App(Message::TogglePopup)
                });
            }
            Message::ProcessSelectFolder(folder) => {
                let video_selected = self.video_entity == self.download_type.active();
                if video_selected {
                    self.video_folder = folder;
                } else {
                    self.audio_folder = folder;
                }
                return Task::done(Action::App(Message::TogglePopup));
            }
            Message::CancelDownload(id) => {
                if let Some(cancel_tx) = self.cancel_senders.remove(&id) {
                    let _ = cancel_tx.send(());
                }
                self.active_downloads.retain(|d| d.id != id);
                let mut notify = Notification::new()
                    .appname("yt-dlp applet")
                    .icon("multimedia-video-player-symbolic")
                    .finalize();
                tokio::spawn(async move {
                    let _ = notify.summary(&fl_str!("download-cancelled", title = "")).show_async().await;
                });
            }
            Message::Download => {
                let video_selected = self.video_entity == self.download_type.active();

                let download_id = self.next_download_id;
                self.next_download_id += 1;

                self.active_downloads.push(ActiveDownload {
                    id: download_id,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    eta_secs: None,
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    is_post_processing: false,
                    is_audio: !video_selected,
                    stage: DownloadStage::Preparing,
                    playlist_current: None,
                    playlist_total: None,
                    playlist_title: None,
                });

                let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();
                self.cancel_senders.insert(download_id, cancel_tx);

                let url = self.url.clone();
                let custom_name = if self.custom_name.trim().is_empty() {
                    None
                } else {
                    Some(self.custom_name.clone())
                };
                let lib_dir = self.lib_dir.clone();
                self.url.clear();
                self.custom_name.clear();
                let output_dir = PathBuf::from(if video_selected {
                    &self.video_folder
                } else {
                    &self.audio_folder
                });
                let output_dir_ref = output_dir.clone();
                let video_container_ext = self.video_container.extension();
                let audio_ext = self.audio_codec.extension();
                let video_quality = self.video_quality;
                let audio_quality = self.audio_quality;
                let economy_mode = self.economy_mode;
                let subtitle_mode = self.subtitle_mode;

                return Task::stream(cosmic::iced::stream::channel(
                    64,
                    move |mut output: cosmic::iced::futures::channel::mpsc::Sender<Action<Message>>| async move {
                        let notify = Notification::new()
                            .appname("yt-dlp applet")
                            .icon("multimedia-video-player-symbolic")
                            .finalize();

                        let downloader =
                            fetcher::with_output_dir(&lib_dir, output_dir).await;

                        run_download_job(
                            download_id,
                            video_selected,
                            &url,
                            custom_name,
                            video_container_ext,
                            &downloader,
                            &output_dir_ref,
                            audio_ext,
                            video_quality,
                            audio_quality,
                            economy_mode,
                            subtitle_mode,
                            &mut cancel_rx,
                            &mut output,
                            notify,
                        ).await;
                    },
                ));
            }
            Message::DownloadProgress {
                id,
                percent,
                speed_mbps,
                eta_secs,
                downloaded_bytes,
                total_bytes,
                is_post_processing,
            } => {
                if let Some(dl) = self.active_downloads.iter_mut().find(|d| d.id == id) {
                    dl.percent = percent;
                    dl.speed_mbps = speed_mbps;
                    dl.eta_secs = eta_secs;
                    dl.downloaded_bytes = downloaded_bytes;
                    dl.total_bytes = total_bytes;
                    dl.is_post_processing = is_post_processing;
                }
            }
            Message::DownloadStage { id, stage } => {
                if let Some(dl) = self.active_downloads.iter_mut().find(|d| d.id == id) {
                    dl.stage = stage;
                    dl.is_post_processing = stage == DownloadStage::PostProcessing;
                }
            }
            Message::PlaylistProgress { id, current, total, video_title } => {
                if let Some(dl) = self.active_downloads.iter_mut().find(|d| d.id == id) {
                    dl.playlist_current = Some(current);
                    dl.playlist_total = Some(total);
                    if !video_title.is_empty() {
                        dl.playlist_title = Some(video_title);
                    }
                    dl.percent = 0.0;
                    dl.downloaded_bytes = 0;
                    dl.total_bytes = 0;
                    dl.eta_secs = None;
                    dl.is_post_processing = false;
                }
            }
            Message::Finished(id) => {
                self.cancel_senders.remove(&id);
                self.active_downloads.retain(|d| d.id != id);
            }
            Message::SurfaceAction(action) => {
                return cosmic::surface::surface_task(action);
            }
            Message::TogglePlatforms => {
                self.show_platforms = !self.show_platforms;
            }
            Message::ToggleAbout => {
                self.show_about = !self.show_about;
            }
            Message::ToggleEconomyMode => {
                self.economy_mode = !self.economy_mode;
            }
            Message::SubtitleModeSelected(idx) => {
                if let Some(&mode) = SUBTITLE_MODES.get(idx) {
                    self.subtitle_mode = mode;
                }
            }
            Message::CheckForUpdates => {
                self.is_checking_updates = true;
                return Task::perform(check_for_updates(), |result| {
                    Action::App(Message::UpdateCheckResult(result))
                });
            }
            Message::UpdateCheckResult(result) => {
                self.is_checking_updates = false;
                self.update_available = None;
                let current_version = format!("v{}", env!("CARGO_PKG_VERSION"));
                match result {
                    Ok(Some(release)) => {
                        self.update_last_msg = Some(format!(
                            "{} {} → {}",
                            fl!("update-available"),
                            current_version,
                            release.tag_name
                        ));
                        self.update_available = Some(release);
                    }
                    Ok(None) => {
                        self.update_last_msg = Some(fl_str!(
                            "no-updates",
                            version = current_version
                        ).to_string());
                    }
                    Err(e) => {
                        self.update_last_msg = Some(format!("Erro: {e}"));
                    }
                }
            }
            Message::InstallUpdate => {
                if let Some(ref release) = self.update_available {
                    self.is_installing_update = true;
                    let download_url = release.download_url.clone();
                    return Task::perform(install_update(download_url), |result| {
                        Action::App(Message::InstallResult(result))
                    });
                }
            }
            Message::InstallResult(result) => {
                self.is_installing_update = false;
                match result {
                    Ok(()) => {
                        return Task::perform(restart_applet(), |error| {
                            Action::App(Message::RestartFailed(error))
                        });
                    }
                    Err(e) => {
                        tokio::spawn(async move {
                            let mut binding = Notification::new();
                            let notify = binding
                                .appname("yt-dlp applet")
                                .summary("Erro na instalação")
                                .body(&e);
                            let _ = notify.show_async().await;
                        });
                    }
                }
            }
            Message::RestartFailed(error) => {
                self.update_last_msg = Some(format!("Atualizado, mas não foi possível reiniciar: {error}"));
                tokio::spawn(async move {
                    let mut binding = Notification::new();
                    let notify = binding
                        .appname("yt-dlp applet")
                        .summary("Atualização instalada")
                        .body("Reabra o applet no painel para usar a nova versão.");
                    let _ = notify.show_async().await;
                });
            }
        }
        Task::none()
    }
}

// ---------------------------------------------------------------------------
// View helpers
// ---------------------------------------------------------------------------

impl Ytdlp {
    fn view_about(&self) -> Element<'_, Message> {
        let Spacing {
            space_xxs, space_xs, space_s, ..
        } = cosmic::theme::active().cosmic().spacing;

        column![
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
            row![
                cosmic::widget::icon::from_name("Dixycat-yt-dlp-icon")
                    .size(64)
                    .icon(),
                column![
                    cosmic::widget::text::title4("Dixycat-ext-cosmic-yt-dlp"),
                    cosmic::widget::text::body(fl!("about-summary")),
                    cosmic::widget::text::caption(fl!("about-license")),
                ]
                .spacing(space_xxs)
                .width(Length::Fill),
            ]
            .spacing(space_s)
            .align_y(Alignment::Center)
            .apply(padded_control),
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
        ]
        .spacing(space_xs)
        .into()
    }

    /// Renders the expandable list of supported video & audio platforms.
    fn view_platforms(&self) -> Element<'_, Message> {
        let Spacing {
            space_xxs, space_xs, space_s, ..
        } = cosmic::theme::active().cosmic().spacing;

        column![
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
            row![
                cosmic::widget::text::body("▶ YouTube"),
                cosmic::widget::text::body("📸 Instagram"),
            ]
            .spacing(space_s)
            .apply(padded_control),
            row![
                cosmic::widget::text::body("𝕏 Twitter/X"),
                cosmic::widget::text::body("🟣 Twitch"),
                cosmic::widget::text::body("🟠 SoundCloud"),
            ]
            .spacing(space_s)
            .apply(padded_control),
            row![
                cosmic::widget::text::body("🔵 Facebook"),
                cosmic::widget::text::body("🔴 Reddit"),
            ]
            .spacing(space_s)
            .apply(padded_control),
            cosmic::widget::text::caption(fl!("platforms-footer"))
                .apply(padded_control),
            padded_control(divider::horizontal::default()).padding([space_xxs, space_s]),
        ]
        .spacing(space_xs)
        .into()
    }

    fn view_video(&self, popup_id: Option<window::Id>) -> Element<'_, Message> {
        let container_idx = VIDEO_CONTAINERS
            .iter()
            .position(|&c| c == self.video_container);
        let video_quality_idx = VIDEO_QUALITIES
            .iter()
            .position(|&q| q == self.video_quality);
        let video_codec_idx = VIDEO_CODECS.iter().position(|&c| c == self.video_codec);

        let Spacing { space_xxs, .. } = cosmic::theme::active().cosmic().spacing;

        let container_dropdown: Element<'_, Message> = if let Some(pid) = popup_id {
            Element::from(
                cosmic::widget::dropdown::popup_dropdown(
                    VIDEO_CONTAINER_LABELS,
                    container_idx,
                    Message::VideoContainerSelected,
                    pid,
                    Message::SurfaceAction,
                    |m| m,
                )
                .width(Length::FillPortion(1)),
            )
        } else {
            Element::from(
                cosmic::widget::dropdown(
                    VIDEO_CONTAINER_LABELS,
                    container_idx,
                    Message::VideoContainerSelected,
                )
                .width(Length::FillPortion(1)),
            )
        };

        let quality_dropdown: Element<'_, Message> = if let Some(pid) = popup_id {
            Element::from(
                cosmic::widget::dropdown::popup_dropdown(
                    VIDEO_QUALITY_LABELS,
                    video_quality_idx,
                    Message::VideoQualitySelected,
                    pid,
                    Message::SurfaceAction,
                    |m| m,
                )
                .width(Length::FillPortion(1)),
            )
        } else {
            Element::from(
                cosmic::widget::dropdown(
                    VIDEO_QUALITY_LABELS,
                    video_quality_idx,
                    Message::VideoQualitySelected,
                )
                .width(Length::FillPortion(1)),
            )
        };

        let codec_dropdown: Element<'_, Message> = if let Some(pid) = popup_id {
            Element::from(
                cosmic::widget::dropdown::popup_dropdown(
                    VIDEO_CODEC_LABELS,
                    video_codec_idx,
                    Message::VideoCodecSelected,
                    pid,
                    Message::SurfaceAction,
                    |m| m,
                )
                .width(Length::FillPortion(1)),
            )
        } else {
            Element::from(
                cosmic::widget::dropdown(
                    VIDEO_CODEC_LABELS,
                    video_codec_idx,
                    Message::VideoCodecSelected,
                )
                .width(Length::FillPortion(1)),
            )
        };

        let subtitle_idx = SUBTITLE_MODES
            .iter()
            .position(|&m| m == self.subtitle_mode);

        let subtitle_dropdown: Element<'_, Message> = if let Some(pid) = popup_id {
            Element::from(
                cosmic::widget::dropdown::popup_dropdown(
                    &self.subtitle_labels,
                    subtitle_idx,
                    Message::SubtitleModeSelected,
                    pid,
                    Message::SurfaceAction,
                    |m| m,
                )
                .width(Length::FillPortion(1)),
            )
        } else {
            Element::from(
                cosmic::widget::dropdown(
                    &self.subtitle_labels,
                    subtitle_idx,
                    Message::SubtitleModeSelected,
                )
                .width(Length::FillPortion(1)),
            )
        };

        column![
            row![
                body(fl!("video-format")).width(Length::FillPortion(1)),
                container_dropdown,
            ]
            .align_y(Alignment::Center)
            .spacing(space_xxs)
            .apply(padded_control),
            row![
                body(fl!("video-quality")).width(Length::FillPortion(1)),
                quality_dropdown,
            ]
            .align_y(Alignment::Center)
            .spacing(space_xxs)
            .apply(padded_control),
            row![
                body(fl!("video-codec")).width(Length::FillPortion(1)),
                codec_dropdown,
            ]
            .align_y(Alignment::Center)
            .spacing(space_xxs)
            .apply(padded_control),
            row![
                body(fl!("subtitle")).width(Length::FillPortion(1)),
                subtitle_dropdown,
            ]
            .align_y(Alignment::Center)
            .spacing(space_xxs)
            .apply(padded_control),
        ]
        .into()
    }


    fn view_audio(&self, popup_id: Option<window::Id>) -> Element<'_, Message> {
        let audio_quality_idx = AUDIO_QUALITIES
            .iter()
            .position(|&q| q == self.audio_quality);
        let audio_codec_idx = AUDIO_CODECS.iter().position(|&c| c == self.audio_codec);

        let Spacing { space_xxs, .. } = cosmic::theme::active().cosmic().spacing;

        let quality_dropdown: Element<'_, Message> = if let Some(pid) = popup_id {
            Element::from(
                cosmic::widget::dropdown::popup_dropdown(
                    AUDIO_QUALITY_LABELS,
                    audio_quality_idx,
                    Message::AudioQualitySelected,
                    pid,
                    Message::SurfaceAction,
                    |m| m,
                )
                .width(Length::FillPortion(1)),
            )
        } else {
            Element::from(
                cosmic::widget::dropdown(
                    AUDIO_QUALITY_LABELS,
                    audio_quality_idx,
                    Message::AudioQualitySelected,
                )
                .width(Length::FillPortion(1)),
            )
        };

        let codec_dropdown: Element<'_, Message> = if let Some(pid) = popup_id {
            Element::from(
                cosmic::widget::dropdown::popup_dropdown(
                    AUDIO_CODEC_LABELS,
                    audio_codec_idx,
                    Message::AudioCodecSelected,
                    pid,
                    Message::SurfaceAction,
                    |m| m,
                )
                .width(Length::FillPortion(1)),
            )
        } else {
            Element::from(
                cosmic::widget::dropdown(
                    AUDIO_CODEC_LABELS,
                    audio_codec_idx,
                    Message::AudioCodecSelected,
                )
                .width(Length::FillPortion(1)),
            )
        };

        column![
            row![
                body(fl!("audio-codec")).width(Length::FillPortion(1)),
                codec_dropdown,
            ]
            .align_y(Alignment::Center)
            .spacing(space_xxs)
            .apply(padded_control),
            row![
                body(fl!("audio-quality")).width(Length::FillPortion(1)),
                quality_dropdown,
            ]
            .align_y(Alignment::Center)
            .spacing(space_xxs)
            .apply(padded_control),
        ]
        .into()
    }

    fn view_progress<'a>(&self, dl: &'a ActiveDownload) -> Element<'a, Message> {
        let Spacing {
            space_xxs, space_xs, space_s, ..
        } = cosmic::theme::active().cosmic().spacing;

        // Playlist header line: "Playlist 3/15 – Some Video Title"
        let playlist_line = if let (Some(cur), Some(tot)) =
            (dl.playlist_current, dl.playlist_total)
        {
            let title = dl.playlist_title.clone().unwrap_or_default();
            Some(fl!("playlist-downloading",
                current = cur,
                total = tot,
                title = title
            ))
        } else {
            None
        };

        let status_line = match dl.stage {
            DownloadStage::Preparing => fl!("preparing-download"),
            DownloadStage::Subtitles => fl!("downloading-subtitles"),
            DownloadStage::PostProcessing => {
                if dl.is_audio {
                    fl!("post-processing-audio")
                } else {
                    fl!("post-processing")
                }
            }
            DownloadStage::Downloading => {
            let eta_text = match dl.eta_secs {
                Some(secs) => {
                    let mins = secs / 60;
                    let s = secs % 60;
                    if mins > 0 {
                        fl!("eta-mins-secs", mins = format!("{:02}", mins), secs = format!("{:02}", s))
                    } else {
                        fl!("eta-secs", secs = s)
                    }
                }
                None => fl!("calculating"),
            };
            format!(
                "{:.1} MB/s  ──  {:.0}%  ──  {}",
                dl.speed_mbps, dl.percent, eta_text
            )
            }
        };

        // Format file size: "1.2 MB / 45.6 MB" or "1.2 MB"
        let size_text = if dl.total_bytes > 0 {
            let dl_mb = dl.downloaded_bytes as f64 / 1_048_576.0;
            let total_mb = dl.total_bytes as f64 / 1_048_576.0;
            format!("{:.1} MB / {:.1} MB", dl_mb, total_mb)
        } else if dl.downloaded_bytes > 0 {
            let dl_mb = dl.downloaded_bytes as f64 / 1_048_576.0;
            format!("{:.1} MB", dl_mb)
        } else {
            String::new()
        };

        let cancel_btn = cosmic::widget::tooltip(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name("process-stop-symbolic")
            )
            .on_press(Message::CancelDownload(dl.id)),
            cosmic::widget::text::body(fl!("cancel")),
            cosmic::widget::tooltip::Position::Left,
        );

        let mut col = column![].spacing(space_xxs);

        if let Some(pl_line) = playlist_line {
            col = col.push(cosmic::widget::text::caption(pl_line));
        }

        col = col
            .push(
                row![
                    column![
                        cosmic::widget::text::caption(status_line),
                        cosmic::widget::determinate_linear(dl.percent / 100.0)
                            .width(Length::Fill)
                            .girth(6),
                        if !size_text.is_empty() {
                            cosmic::widget::text::caption(size_text)
                        } else {
                            cosmic::widget::text::caption("")
                        }
                    ]
                    .spacing(space_xxs)
                    .width(Length::Fill),
                    cancel_btn,
                ]
                .align_y(Alignment::Center)
                .spacing(space_xs),
            );

        col.padding([0, space_s]).into()
    }
}

// ---------------------------------------------------------------------------
// Update checking and installation functions
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::is_version_newer;

    #[test]
    fn semantic_version_compare_handles_major_minor_and_patch() {
        assert!(is_version_newer("v0.10.0", "v0.9.9"));
        assert!(is_version_newer("v1.2.3", "v1.2.2"));
        assert!(!is_version_newer("v0.4.2", "v0.4.2"));
        assert!(!is_version_newer("v1.2.3", "v1.2.4"));
    }
}

/// Checks GitHub releases for updates
async fn check_for_updates() -> Result<Option<ReleaseInfo>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get("https://api.github.com/repos/felix-the-cat177/cosmic-applet-yt-dlp-dixycat/releases/latest")
        .header("User-Agent", "cosmic-applet-yt-dlp")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }
    
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let tag_name = json["tag_name"]
        .as_str()
        .unwrap_or("v0.0.0")
        .to_string();
    
    let current_version = env!("CARGO_PKG_VERSION");
    let current_tag = format!("v{}", current_version);

    // Ignore release tags equal to or older than the app version. This avoids false
    // positives when the user is running a locally packaged build that is newer than
    // the last public GitHub release.
    if tag_name == current_tag || !is_version_newer(&tag_name, &current_tag) {
        return Ok(None);
    }
    
    let version = tag_name.trim_start_matches('v').to_string();
    let release_notes = json["body"]
        .as_str()
        .unwrap_or("")
        .lines()
        .take(5)
        .collect::<Vec<_>>()
        .join("\n");
    
    // Find the .deb download URL for the current architecture
    let assets = json["assets"]
        .as_array()
        .ok_or("No assets found in release")?;
    
    let arch = std::env::consts::ARCH;
    let deb_suffix = match arch {
        "x86_64" => "amd64.deb",
        "aarch64" => "arm64.deb",
        _ => "amd64.deb",
    };
    
    let download_url = assets
        .iter()
        .find(|asset| {
            asset["name"]
                .as_str()
                .map(|name| name.ends_with(deb_suffix))
                .unwrap_or(false)
        })
        .and_then(|asset| asset["browser_download_url"].as_str())
        .ok_or("No .deb asset found for this architecture")?
        .to_string();
    
    Ok(Some(ReleaseInfo {
        version,
        tag_name,
        download_url,
        release_notes,
    }))
}

/// Downloads and installs update using polkit
async fn install_update(download_url: String) -> Result<(), String> {
    use std::io::Write;
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;
    
    // Save to temporary location
    let temp_dir = std::env::temp_dir();
    let deb_path = temp_dir.join("cosmic-applet-yt-dlp-update.deb");
    
    let mut file = std::fs::File::create(&deb_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    
    drop(file);
    
    // Use polkit to install with elevated privileges
    let deb_path_str = deb_path.to_string_lossy();
    let output = tokio::process::Command::new("pkexec")
        .arg("apt")
        .arg("install")
        .arg("-y")
        .arg(&*deb_path_str)
        .output()
        .await
        .map_err(|e| format!("Failed to execute pkexec: {}. Make sure polkit is installed.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Installation failed: {}", stderr));
    }
    
    // Clean up temp file
    let _ = std::fs::remove_file(&deb_path);
    
    Ok(())
}

/// Replaces this process with the updated executable. `exec` preserves the
/// process identity used by COSMIC, unlike `exit(0)`, which removes the applet
/// from the panel without starting it again.
async fn restart_applet() -> String {
    let mut binding = Notification::new();
    let notify = binding
        .appname("yt-dlp applet")
        .summary("Atualização instalada")
        .body("Reiniciando o applet com a nova versão...");
    let _ = notify.show_async().await;

    // Give the desktop notification a moment to be delivered before replacing
    // the process. The new executable keeps the original command-line flags.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => return format!("não foi possível localizar o executável: {error}"),
        };
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        debug_log!("Restarting applet with executable {:?}", executable);

        let error = std::process::Command::new(executable).args(args).exec();
        format!("não foi possível iniciar a nova versão: {error}")
    }

    #[cfg(not(unix))]
    {
        "reinício automático não é suportado neste sistema".to_string()
    }
}
