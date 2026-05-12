use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// Test that otree checks file size before reading the file into memory.
/// This prevents OOM errors when users accidentally try to open very large files.
///
/// The default max_data_size is 30 MiB. This test creates a file larger than that
/// and verifies that otree rejects it with a file size error BEFORE attempting to
/// read the entire file into memory.
#[test]
fn test_rejects_large_file_without_oom() {
    let temp_dir = TempDir::new().unwrap();
    let large_file = temp_dir.path().join("large.json");

    // Create a file larger than default max_data_size (30 MiB)
    // We'll create a 35 MiB file with valid JSON
    let target_size: usize = 35 * 1024 * 1024;

    // Write a large but valid JSON array
    let mut file = fs::File::create(&large_file).unwrap();
    write!(file, "[").unwrap();

    // Write enough objects to exceed 30 MiB
    // Each object is ~39 bytes: {"id":123,"name":"test","value":true},
    let obj = r#"{"id":123,"name":"test","value":true},"#;
    let obj_size = obj.len();
    let num_objects = target_size / obj_size;

    for _ in 0..num_objects {
        write!(file, "{}", obj).unwrap();
    }
    write!(file, "{{}}]").unwrap(); // Close the array with a final object
    file.flush().unwrap();
    drop(file);

    // Verify file was created with expected size
    let metadata = fs::metadata(&large_file).unwrap();
    assert!(
        metadata.len() > (30 * 1024 * 1024),
        "Test file should be larger than 30 MiB, got {} bytes",
        metadata.len()
    );

    // Run otree binary with the large file using --to conversion
    // (--to doesn't require a TTY, so we can test it)
    // It should reject the file with a size error, not OOM
    let output = Command::new(env!("CARGO_BIN_EXE_otree"))
        .arg(large_file.to_str().unwrap())
        .arg("--to")
        .arg("yaml")
        .arg("--ignore-config")
        .output()
        .expect("Failed to run otree");

    // Should fail with an error about file size
    assert!(
        !output.status.success(),
        "otree should reject the large file"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("too large"),
        "Error message should mention file size being too large, got: {}",
        stderr
    );
    assert!(
        stderr.contains("MiB"),
        "Error message should show sizes in MiB, got: {}",
        stderr
    );

    // The key improvement: the process should reject the file based on metadata check
    // BEFORE attempting to read 35 MiB into memory, preventing OOM on truly large files
}
