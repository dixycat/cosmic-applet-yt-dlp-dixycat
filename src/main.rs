// SPDX-License-Identifier: GPL-3.0-only

mod applet;
mod fetcher;
mod formats;
mod i18n;

fn main() -> cosmic::iced::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--debug" || a == "-d") || std::env::var("COSMIC_YTDLP_DEBUG").is_ok() {
        applet::enable_debug();
    }

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    let flags = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(fetcher::binaries());

    cosmic::applet::run::<applet::Ytdlp>(flags)
}
