use maud::{DOCTYPE, Markup, PreEscaped, html};

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
                script src="/static/upload.js" defer="defer" {}
            }
            body {
                header class="site-header" {
                    div class="header-container" {
                        a href="/" class="logo" {
                            svg
                                width="20"
                                height="20"
                                viewBox="0 0 512 512"
                                fill="currentColor"
                                aria-hidden="true"
                            {
                                path d="M372.87,33.391c-46.903,0-90.88,23.598-116.87,62.152c-25.99-38.555-69.967-62.152-116.87-62.152C62.413,33.391,0,95.804,0,172.522c0,37.935,14.164,73.011,39.88,98.76l200.38,200.804c4.207,4.207,9.794,6.522,15.74,6.522s11.532-2.315,15.74-6.521l200.314-200.772C497.815,245.522,512,210.435,512,172.522C512,95.804,449.587,33.391,372.87,33.391z" {}
                            }
                            span { "avafiles" }
                        }
                        nav class="nav-links" {
                            a href="/about" { "About" }
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

pub async fn about_page() -> Markup {
    base_layout(
        "avafiles • about",
        html! {
            div class="about-card" {
                div class="card-header" {
                    h1 { "avafiles" }
                }
                div class="card-body" {
                    p { "A simple, fast audio uploader. Drop in your files and get them hosted instantly." }
                }
            }
        },
    )
}
