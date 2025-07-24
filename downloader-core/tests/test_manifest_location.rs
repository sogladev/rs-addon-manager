#[cfg(test)]
mod tests {
    use downloader_core::manifest::*;

    #[test]
    fn invalid_url() {
        let result = Url::parse("not-a-url".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_url_scheme() {
        let result = Url::parse("127.0.0.1:8080/manifest.json".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn valid_url_http() {
        let result = Url::parse("http://localhost:8080/manifest.json".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn valid_url_https() {
        let result = Url::parse("https://localhost:8080/manifest.json".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_unix_path() {
        let result = Url::parse("/non/existent/path".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_windows_path() {
        let result = Url::parse("C://non//existent//file.txt".to_string());
        assert!(result.is_err());
    }
}
