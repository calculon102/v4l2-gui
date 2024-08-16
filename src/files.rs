use std::fs;

pub fn _get_video_devices(dir: &str) -> Vec<String> {
    let mut video_files = Vec::new();

    // Read the entries in the specified directory
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Check if the path starts with "/dev/video"
                if let Some(path_str) = path.to_str() {
                    if path_str.starts_with(&format!("{}/video", dir)) {
                        video_files.push(path_str.to_string());
                    }
                }
            }
        }
    }

    video_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_get_video_devices() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // Create mock files
        let video_file_0 = dir_path.join("video0");
        let video_file_1 = dir_path.join("video1");
        let non_video_file = dir_path.join("audio0");

        File::create(&video_file_0).unwrap();
        File::create(&video_file_1).unwrap();
        File::create(&non_video_file).unwrap();

        let mut result = _get_video_devices(dir_path.to_str().unwrap());

        // Convert paths to strings for easier comparison
        result.sort();
        let expected: Vec<String> = vec![
            video_file_0.to_str().unwrap().to_string(),
            video_file_1.to_str().unwrap().to_string(),
        ];

        assert_eq!(result, expected);
    }
}
