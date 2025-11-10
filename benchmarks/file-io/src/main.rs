//! BENCH-007: File I/O
//!
//! Workload Type: I/O-bound (system call overhead)
//! Expected Result: 104,857,600 bytes (100 MB)
//! Expected Runtime: ~500ms

use std::fs::File;
use std::io::{Read, Write};
use std::time::Instant;

/// Benchmark file I/O: write 100MB, then read it back
fn benchmark_io() -> std::io::Result<usize> {
    let data = vec![42u8; 100 * 1024 * 1024]; // 100 MB

    // Write phase
    let mut file = File::create("/tmp/benchmark_io.dat")?;
    file.write_all(&data)?;
    drop(file);

    // Read phase
    let mut file = File::open("/tmp/benchmark_io.dat")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Clean up
    std::fs::remove_file("/tmp/benchmark_io.dat").ok();

    Ok(buffer.len())
}

fn main() {
    let t0 = Instant::now();
    let t1 = Instant::now();
    let result = benchmark_io().unwrap();
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_file_io_small() {
        let data = vec![1u8; 1024]; // 1 KB
        let path = "/tmp/test_benchmark_io_small.dat";

        let mut file = File::create(path).unwrap();
        file.write_all(&data).unwrap();
        drop(file);

        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 1024);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_file_io_medium() {
        let data = vec![42u8; 1024 * 1024]; // 1 MB
        let path = "/tmp/test_benchmark_io_medium.dat";

        let mut file = File::create(path).unwrap();
        file.write_all(&data).unwrap();
        drop(file);

        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 1024 * 1024);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_file_io_target_workload() {
        // Test with 100 MB
        let result = benchmark_io().unwrap();
        assert_eq!(result, 100 * 1024 * 1024); // 104,857,600
    }
}
