use std::path::Path;
use anyhow::{Context, Result};

pub struct FileInfo {
    pub name: String,
    pub size: u64,
}

pub fn check_file(path: &str) -> Result<FileInfo> {
    let file_path = Path::new(path);

    // Check whether the file exists
    if !file_path.exists() {
        anyhow::bail!("File does not exist: {}", path);
    }

    // Check whether the path points to a file
    if !file_path.is_file() {
        anyhow::bail!("The given path is not a file: {}", path);
    }

    // Check whether the file can be opened for reading
    let _file = std::fs::File::open(file_path)
        .context("Could not open file for reading")?;

    // Get file metadata
    let metadata = std::fs::metadata(file_path)
        .context("Could not read file metadata")?;

    // Get file name
    let name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("Could not read file name")?
        .to_string();

    Ok(FileInfo {
        name,
        size: metadata.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_file() {
        let result = check_file("Cargo.toml");

        assert!(result.is_ok());

        let info = result.unwrap();

        assert_eq!(info.name, "Cargo.toml");
        assert!(info.size > 0);
    }

    #[test]
    fn test_file_does_not_exist() {
        let result = check_file("this_file_does_not_exist.txt");

        assert!(result.is_err());
    }

    #[test]
    fn test_path_is_directory() {
        let result = check_file("src");

        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file() {
        let test_path = "test_empty_file.txt";

        // Create an empty test file
        std::fs::write(test_path, "").unwrap();

        let result = check_file(test_path);

        assert!(result.is_ok());

        let info = result.unwrap();

        assert_eq!(info.name, test_path);
        assert_eq!(info.size, 0);

        // Remove the test file
        std::fs::remove_file(test_path).unwrap();
    }
}
