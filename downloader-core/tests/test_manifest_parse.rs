use downloader_core::manifest::Manifest;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manifest_from_json() {
        // Prepare valid manifest JSON content
        let json_content = r#"
        {
            "Version": "1.0",
            "Uid": "5a63cd8c-956c-48a0-95ae-7e41d1e73182",
            "Files": [
                {
                    "Path": "files/A.bin",
                    "Hash": "b6d81b360a5672d80c27430f39153e2c",
                    "Size": 1048576,
                    "Custom": true,
                    "Urls": {
                        "cloudflare": "http://localhost:8080/files/A.bin",
                        "digitalocean": "http://localhost:8080/files/A.bin",
                        "none": "http://localhost:8080/files/A.bin"
                    }
                }
            ],
            "Removals": [
              "old/deprecated_file.txt",
              "old/another_file.dll"
            ]
        }
        "#;

        // Deserialize manifest from JSON string
        let manifest: Manifest = serde_json::from_str(json_content)
            .expect("Failed to parse manifest JSON");
        assert_eq!(manifest.version, "1.0");
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].path, "files/A.bin");
        let removals = manifest.removals.unwrap();
        assert_eq!(removals.len(), 2);
        assert_eq!(removals[0], "old/deprecated_file.txt");
        assert_eq!(removals[1], "old/another_file.dll");
    }

    #[tokio::test]
    async fn test_manifest_deserialize_invalid_json() {
        // Prepare invalid JSON content
        let invalid_json = "invalid json";

        // Expect deserialization to error out on invalid JSON
        let result: Result<Manifest, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}