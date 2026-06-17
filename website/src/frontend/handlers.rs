use crate::{error::ApiError, state::AppState, uploads::queries};
use axum::extract::{Path, State};
use maud::{DOCTYPE, Markup, html};

pub fn base_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="crossorigin";
                link href="https://fonts.googleapis.com/css2?family=Inter:ital,wght@0,400;0,500;0,600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/static/style.css";
            }
            body {
                header class="site-header" {
                    div class="header-container" {
                        a href="/" class="logo" {
                            span { "avafiles" }
                        }
                        nav class="nav-links" {
                            a href="/about" { "about" }
                        }
                    }
                }
                main class="main-content" {
                    (content)
                }
                footer class="site-footer" {
                    div class="footer-container" {
                        div class="footer-left" {
                            a href="/" { "avafiles.cc" }
                        }
                        div class="footer-right" {
                            a href="/privacy" { "privacy" }
                            a href="/terms" { "terms" }
                        }
                    }
                }
            }
        }
    }
}

pub async fn home_page() -> Markup {
    base_layout(
        "avafiles",
        html! {
            script src="/static/upload.js" defer="defer" {}
            div class="upload-page" {
                h1 class="page-title" { "avafiles" }
                p class="page-subtitle" { "Upload and share audio files" }

                div class="upload-stack" {
                    div id="drop-zone" class="drop-zone" {
                        input type="file" id="file-input" accept=".mp3,.wav,.flac,.m4a" hidden="hidden";
                        p class="drop-text" {
                            "drop files here or "
                            span class="browse-text" { "browse" }
                        }
                        span class="format-hint" { "mp3, wav, flac, m4a • 200 MB" }
                    }

                    div id="uploading-item" class="upload-item uploading-item hidden" {
                        span id="uploading-name" class="upload-item-link" {}
                        span id="uploading-size" class="upload-item-meta" {}
                    }

                    div id="uploads-list" class="uploads-list" {}

                    p id="status-text" class="status-text" aria-live="polite" {}
                }
            }
        },
    )
}

fn is_audio_file(name: &str) -> bool {
    match name.rsplit('.').next().map(|s| s.to_ascii_lowercase()) {
        Some(ext) => matches!(
            ext.as_str(),
            "mp3" | "wav" | "flac" | "m4a" | "aac" | "ogg" | "opus"
        ),
        None => false,
    }
}

pub async fn file_page(
    State(state): State<AppState>,
    Path(upload_id): Path<String>,
) -> Result<Markup, ApiError> {
    let Some(file) = queries::find_file_upload_by_short_code(&state.db, &upload_id).await? else {
        return Err(ApiError::NotFound);
    };

    // Helper: format bytes to human readable
    fn format_file_size(bytes: i64) -> String {
        if bytes == 0 {
            return "0 Bytes".into();
        }
        let k = 1024.0_f64;
        let sizes = ["Bytes", "KB", "MB", "GB"];
        let i = ((bytes as f64).ln() / k.ln()) as usize;
        let i = i.min(sizes.len() - 1);
        format!("{:.2} {}", (bytes as f64) / k.powi(i as i32), sizes[i])
    }

    fn format_date(dt: time::OffsetDateTime) -> String {
        format!(
            "{month:02}/{day:02}/{year}",
            year = dt.year(),
            month = dt.month() as u8,
            day = dt.day()
        )
    }

    let download_url = format!("/api/uploads/{upload_id}/download");
    let is_audio = is_audio_file(&file.original_name);

    let page = if is_audio {
        let file_size = format_file_size(file.size);
        let created_at = format_date(file.created_at);

        html! {
                div class="audio-page" {
                    div class="audio-card" {
                        div class="cover-art" {
                            svg xmlns="http://www.w3.org/2000/svg" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5" {
                                path stroke-linecap="round" stroke-linejoin="round"
                                    d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" {}
                            }
                        }

                        div class="audio-info" {
                            span class="audio-meta-item" { (created_at) }
                            span class="audio-meta-sep" { "·" }
                            span class="audio-meta-item" { (file_size) }
                            // span class="audio-meta-sep" { "·" }
                            // span class="audio-meta-item" { "320 kbps" }
                        }

                        div class="audio-header" {
                            h2 class="audio-title" { (file.original_name) }
                            a class="download-btn"
                                href=(download_url)
                                download=(file.original_name)
                                target="_blank"
                                rel="noopener noreferrer"
                                title="Download" {
                                svg class="download-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" {}
                                }
                            }
                        }

                        audio
                            id="audio-player"
                            preload="metadata"
                            src=(download_url.as_str())
                            style="display: none;"
                        {}

                        div class="controls-row" {
                            button id="play-button"
                                    type="button"
                                    class="play-btn"
                                    aria-label="Play" {
                                svg id="play-icon" class="play-icon" fill="currentColor" viewBox="0 0 24 24" {
                                    path d="M8 5v14l11-7z" {}
                                }
                                svg id="pause-icon" class="pause-icon hidden" fill="currentColor" viewBox="0 0 24 24" {
                                    path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" {}
                                }
                            }

                            span id="current-time" class="time-display" { "0:00" }

                            div class="seek-container" {
                                div class="seek-track" {}
                                div id="seek-progress" class="seek-fill" style="width: 0%;" {}
                                input id="seek-bar"
                                    type="range"
                                    min="0"
                                    max="100"
                                    value="0"
                                    class="seek-input";
                            }

                            span id="duration" class="time-display" { "0:00" }

                            div class="volume-container" {
                                button id="volume-button"
                                        type="button"
                                        class="volume-btn"
                                        aria-label="Volume" {
                                    svg fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                            d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" {}
                                    }
                                }
                                div id="volume-popup" class="volume-popup hidden" {
                                    div class="volume-slider-wrap" {
                                        div id="volume-progress" class="volume-fill" {}
                                        input id="volume-slider"
                                              type="range"
                                              min="0"
                                              max="1"
                                              step="0.01"
                                              value="1"
                                              class="volume-input";
                                        div id="volume-thumb" class="volume-thumb" {}
                                    }
                                }
                            }
                        }
                    }
                }

                script src="/static/audioplayer.js" {}
        }
    } else {
        let file_size = format_file_size(file.size);
        html! {
            div class="audio-page" {
                div class="audio-card file-card-layout" {
                    div class="file-icon-box" {
                        svg class="file-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" {}
                        }
                    }
                    div class="audio-main" {
                        h2 class="audio-title" { (file.original_name) }
                        div class="audio-meta" {
                            span { (file_size) }
                        }
                        a class="small-btn" href=(download_url) download=(file.original_name) {
                            "Download"
                        }
                    }
                }
            }
        }
    };

    Ok(base_layout(
        &format!("avafiles • {}", file.original_name),
        page,
    ))
}

pub async fn about_page() -> Markup {
    base_layout(
        "avafiles • about",
        html! {
            div class="about-card" {
                div class="card-header" {
                    h1 { "avafiles" }
                }
                div class="card-body" {
                    p { "about" }
                }
            }
        },
    )
}

pub async fn privacy_page() -> Markup {
    base_layout(
        "avafiles • privacy",
        html! {
            div class="about-card" {
                div class="card-header" {
                    h1 { "avafiles" }
                }
                div class="card-body" {
                    p { "privacy" }
                }
            }
        },
    )
}

pub async fn terms_page() -> Markup {
    base_layout(
        "avafiles • terms",
        html! {
            div class="about-card" {
                div class="card-header" {
                    h1 { "avafiles" }
                }
                div class="card-body" {
                    p { "terms" }
                }
            }
        },
    )
}
