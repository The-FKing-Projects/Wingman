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
    use webkit2gtk::{NetworkError, PolicyError, WebViewExt};

    window.with_webview(move |platform| {
        let webview = platform.inner();
        webview.connect_load_failed(|webview, _, uri, error| {
            if error.matches(NetworkError::Cancelled)
                || error.matches(PolicyError::FrameLoadInterruptedByPolicyChange)
            {
                return false;
            }
            show_retry_page(webview, uri);
            true
        });
        webview.connect_load_failed_with_tls_errors(|webview, uri, _, _| {
            show_retry_page(webview, uri);
            true
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
}
