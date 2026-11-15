use std::cell::{Cell, RefCell};
use std::rc::Rc;

use url::Url;

const START_URL: &str = "https://copilot.com/chat";

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn offline_page(failing_uri: &str) -> String {
    let url = Url::parse(failing_uri)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https"))
        .map_or_else(|| START_URL.to_owned(), String::from);
    include_str!("offline.html").replace("{{RETRY_URL}}", &escape_html(&url))
}

/// WebKitWebView has no is-main-frame flag on load-failed. Require both:
/// 1. a main-frame page load is in progress (`load-changed` Started..Finished)
/// 2. the failing URI matches that main-frame navigation URI
/// so iframe failures during or after Copilot load cannot replace the window.
fn should_show_retry(
    main_frame_loading: bool,
    failing_uri: &str,
    main_navigation_uri: &str,
) -> bool {
    main_frame_loading
        && !main_navigation_uri.is_empty()
        && failing_uri == main_navigation_uri
}

#[cfg(target_os = "linux")]
fn show_retry_page(webview: &webkit2gtk::WebView, failing_uri: &str) {
    use webkit2gtk::WebViewExt;

    let html = offline_page(failing_uri);
    let webview = webview.clone();
    // Defer past the load-failed signal; loading from inside the handler is unsafe.
    glib::idle_add_local_once(move || {
        webview.load_html(&html, None);
    });
}

#[cfg(target_os = "linux")]
pub fn attach(window: &tauri::WebviewWindow, start: Url) -> tauri::Result<()> {
    use webkit2gtk::{LoadEvent, NetworkError, PolicyError, WebViewExt};

    window.with_webview(move |platform| {
        let webview = platform.inner();
        let main_frame_loading = Rc::new(Cell::new(false));
        let main_navigation_uri = Rc::new(RefCell::new(String::new()));

        webview.connect_load_changed({
            let main_frame_loading = main_frame_loading.clone();
            let main_navigation_uri = main_navigation_uri.clone();
            move |webview, event| match event {
                LoadEvent::Started | LoadEvent::Redirected => {
                    if matches!(event, LoadEvent::Started) {
                        main_frame_loading.set(true);
                    }
                    if let Some(uri) = webview.uri() {
                        *main_navigation_uri.borrow_mut() = uri.to_string();
                    }
                }
                LoadEvent::Finished => main_frame_loading.set(false),
                _ => {}
            }
        });

        webview.connect_load_failed({
            let main_frame_loading = main_frame_loading.clone();
            let main_navigation_uri = main_navigation_uri.clone();
            move |webview, _, uri, error| {
                if error.matches(NetworkError::Cancelled)
                    || error.matches(PolicyError::FrameLoadInterruptedByPolicyChange)
                {
                    return false;
                }
                if !should_show_retry(
                    main_frame_loading.get(),
                    uri,
                    &main_navigation_uri.borrow(),
                ) {
                    return false;
                }
                main_frame_loading.set(false);
                show_retry_page(webview, uri);
                true
            }
        });
        webview.connect_load_failed_with_tls_errors({
            let main_frame_loading = main_frame_loading.clone();
            let main_navigation_uri = main_navigation_uri.clone();
            move |webview, uri, _, _| {
                if !should_show_retry(
                    main_frame_loading.get(),
                    uri,
                    &main_navigation_uri.borrow(),
                ) {
                    return false;
                }
                main_frame_loading.set(false);
                show_retry_page(webview, uri);
                true
            }
        });
        webview.load_uri(start.as_str());
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_url_is_escaped() {
        let page = offline_page("https://copilot.com/chat?a=1&b=2");
        assert!(page.contains("href=\"https://copilot.com/chat?a=1&amp;b=2\""));
        assert!(!page.contains("{{RETRY_URL}}"));
    }

    #[test]
    fn unsafe_or_invalid_retry_urls_use_copilot() {
        for uri in ["javascript:alert(1)", "file:///etc/passwd", "not a url"] {
            let page = offline_page(uri);
            assert!(page.contains("href=\"https://copilot.com/chat\""));
        }
    }

    #[test]
    fn retry_requires_main_frame_load_and_matching_uri() {
        let main = "https://copilot.com/chat";
        assert!(should_show_retry(true, main, main));
        assert!(!should_show_retry(false, main, main));
        assert!(!should_show_retry(true, "https://ads.example/iframe", main));
        assert!(!should_show_retry(true, main, ""));
    }
}
