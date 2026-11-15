use std::cell::RefCell;
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

/// `load-failed` can fire for subframe loads. Only treat a failure as top-level when
/// the failing URI matches the main-frame navigation tracked via `load-changed`.
fn is_top_level_failure(failing_uri: &str, main_navigation_uri: &str) -> bool {
    !main_navigation_uri.is_empty() && failing_uri == main_navigation_uri
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
        let main_navigation_uri = Rc::new(RefCell::new(String::new()));

        webview.connect_load_changed({
            let main_navigation_uri = main_navigation_uri.clone();
            move |webview, event| {
                if matches!(event, LoadEvent::Started | LoadEvent::Redirected) {
                    if let Some(uri) = webview.uri() {
                        *main_navigation_uri.borrow_mut() = uri.to_string();
                    }
                }
            }
        });

        webview.connect_load_failed({
            let main_navigation_uri = main_navigation_uri.clone();
            move |webview, _, uri, error| {
                if error.matches(NetworkError::Cancelled)
                    || error.matches(PolicyError::FrameLoadInterruptedByPolicyChange)
                {
                    return false;
                }
                if !is_top_level_failure(uri, &main_navigation_uri.borrow()) {
                    return false;
                }
                show_retry_page(webview, uri);
                true
            }
        });
        webview.connect_load_failed_with_tls_errors({
            let main_navigation_uri = main_navigation_uri.clone();
            move |webview, uri, _, _| {
                if !is_top_level_failure(uri, &main_navigation_uri.borrow()) {
                    return false;
                }
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
    fn only_matching_main_navigation_uri_is_top_level() {
        assert!(is_top_level_failure(
            "https://copilot.com/chat",
            "https://copilot.com/chat"
        ));
        assert!(!is_top_level_failure(
            "https://ads.example/iframe",
            "https://copilot.com/chat"
        ));
        assert!(!is_top_level_failure("https://copilot.com/chat", ""));
    }
}
