use url::Url;

const SITE_HOST: &str = "xiaban.xyz";

#[derive(Debug, PartialEq, Eq)]
pub enum Navigation {
    /// Load it inside the shell.
    Stay,
    /// Hand it to the system (browser, mail client, dialer).
    OpenExternally,
    /// Do nothing.
    Block,
}

pub fn classify(url: &Url) -> Navigation {
    match url.scheme() {
        "tauri" => Navigation::Stay,
        "about" => match url.as_str() {
            "about:blank" | "about:srcdoc" => Navigation::Stay,
            _ => Navigation::Block,
        },
        "mailto" | "tel" => Navigation::OpenExternally,
        "http" | "https" => classify_web(url),
        _ => Navigation::Block,
    }
}

fn classify_web(url: &Url) -> Navigation {
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    // The bundled launcher page is served from this host on Windows.
    if host == "tauri.localhost" {
        return Navigation::Stay;
    }
    let on_site = host == SITE_HOST || host.ends_with(&format!(".{SITE_HOST}"));
    if url.scheme() == "https" && on_site && url.port().is_none() {
        Navigation::Stay
    } else {
        Navigation::OpenExternally
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify_str(url: &str) -> Navigation {
        classify(&Url::parse(url).unwrap())
    }

    #[test]
    fn site_pages_and_subdomains_stay() {
        for url in ["https://xiaban.xyz/", "https://xiaban.xyz/games/snake-arena?from=card#top", "https://m.XIABAN.xyz/x"] {
            assert_eq!(classify_str(url), Navigation::Stay, "{url}");
        }
    }

    #[test]
    fn bundled_launcher_page_stays() {
        for url in ["tauri://localhost/index.html", "http://tauri.localhost/", "https://tauri.localhost/", "about:blank"] {
            assert_eq!(classify_str(url), Navigation::Stay, "{url}");
        }
    }

    #[test]
    fn other_sites_open_in_the_system_browser() {
        for url in ["https://example.com/", "http://example.com/", "mailto:me@xiaban.xyz", "tel:10086"] {
            assert_eq!(classify_str(url), Navigation::OpenExternally, "{url}");
        }
    }

    #[test]
    fn lookalike_hosts_and_plain_http_do_not_count_as_the_site() {
        for url in [
            "https://evilxiaban.xyz/",
            "https://xiaban.xyz.evil.example/",
            "https://xiaban.xyz@evil.example/",
            "https://evil.example/?next=https://xiaban.xyz/",
            "http://xiaban.xyz/",
            "https://xiaban.xyz:8443/",
        ] {
            assert_eq!(classify_str(url), Navigation::OpenExternally, "{url}");
        }
    }

    #[test]
    fn local_files_scripts_and_custom_schemes_are_blocked() {
        for url in ["file:///etc/passwd", "javascript:alert(1)", "data:text/html,hi", "blob:https://xiaban.xyz/x", "about:config", "ms-msdt:/id"] {
            assert_eq!(classify_str(url), Navigation::Block, "{url}");
        }
    }
}
