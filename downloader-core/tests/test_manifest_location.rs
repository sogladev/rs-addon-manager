#[cfg(test)]
mod tests {
    use downloader_core::manifest::*;

    #[test]
    fn invalid_url() {
        let result = Location::parse("not-a-url".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_url_scheme() {
        let result = Location::parse("127.0.0.1:8080/manifest.json".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn valid_url_http() {
        let result = Location::parse("http://localhost:8080/manifest.json".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn valid_url_https() {
        let result = Location::parse("https://localhost:8080/manifest.json".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_unix_path() {
        let result = Location::parse("/non/existent/path".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_windows_path() {
        let result = Location::parse("C://non//existent//file.txt".to_string());
        assert!(result.is_err());
    }
}
