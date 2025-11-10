//! BENCH-004: Quicksort
//!
//! Workload Type: Memory-bound (random access patterns)
//! Expected Result: Sorted array (1M integers)
//! Expected Runtime: ~200ms

use std::time::Instant;

/// Choose median-of-three pivot and move it to end
fn median_of_three(arr: &mut [i32]) {
    let len = arr.len();
    let mid = len / 2;
    let last = len - 1;

    // Sort first, middle, last
    if arr[0] > arr[mid] {
        arr.swap(0, mid);
    }
    if arr[mid] > arr[last] {
        arr.swap(mid, last);
    }
    if arr[0] > arr[mid] {
        arr.swap(0, mid);
    }
    // Now median is at mid, move to end
    arr.swap(mid, last);
}

/// Quicksort implementation with iterative approach to avoid stack overflow
///
/// Time Complexity: O(n log n) average, O(n²) worst case
/// Space Complexity: O(log n) for recursion stack (with tail recursion)
fn quicksort(arr: &mut [i32]) {
    let mut stack = vec![(0, arr.len())];

    while let Some((lo, hi)) = stack.pop() {
        if hi - lo <= 1 {
            continue;
        }

        let pivot_index = {
            let slice = &mut arr[lo..hi];
            if slice.len() >= 3 {
                median_of_three(slice);
            }
            let pivot = slice[slice.len() - 1];
            let mut i = 0;
            for j in 0..slice.len() - 1 {
                if slice[j] <= pivot {
                    slice.swap(i, j);
                    i += 1;
                }
            }
            slice.swap(i, slice.len() - 1);
            lo + i
        };

        // Push larger partition first (for stack space efficiency)
        if pivot_index - lo > hi - pivot_index - 1 {
            stack.push((lo, pivot_index));
            stack.push((pivot_index + 1, hi));
        } else {
            stack.push((pivot_index + 1, hi));
            stack.push((lo, pivot_index));
        }
    }
}

fn main() {
    let mut arr: Vec<i32> = (0..1_000_000).rev().collect();

    let t0 = Instant::now();
    let t1 = Instant::now();
    quicksort(&mut arr);
    let t2 = Instant::now();

    let is_sorted = arr.windows(2).all(|w| w[0] <= w[1]);

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", if is_sorted { 1 } else { 0 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quicksort_empty() {
        let mut arr: Vec<i32> = vec![];
        quicksort(&mut arr);
        assert_eq!(arr, vec![]);
    }

    #[test]
    fn test_quicksort_single_element() {
        let mut arr = vec![42];
        quicksort(&mut arr);
        assert_eq!(arr, vec![42]);
    }

    #[test]
    fn test_quicksort_two_elements() {
        let mut arr = vec![2, 1];
        quicksort(&mut arr);
        assert_eq!(arr, vec![1, 2]);

        let mut arr2 = vec![1, 2];
        quicksort(&mut arr2);
        assert_eq!(arr2, vec![1, 2]);
    }

    #[test]
    fn test_quicksort_small_array() {
        let mut arr = vec![5, 2, 8, 1, 9, 3];
        quicksort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 5, 8, 9]);
    }

    #[test]
    fn test_quicksort_reversed() {
        let mut arr: Vec<i32> = (0..100).rev().collect();
        quicksort(&mut arr);
        let expected: Vec<i32> = (0..100).collect();
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_quicksort_already_sorted() {
        let mut arr: Vec<i32> = (0..100).collect();
        quicksort(&mut arr);
        let expected: Vec<i32> = (0..100).collect();
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_quicksort_with_duplicates() {
        let mut arr = vec![5, 2, 8, 2, 9, 5, 3, 5];
        quicksort(&mut arr);
        assert_eq!(arr, vec![2, 2, 3, 5, 5, 5, 8, 9]);
    }

    #[test]
    fn test_quicksort_target_workload() {
        // Test with a smaller version of the main workload
        let mut arr: Vec<i32> = (0..10_000).rev().collect();
        quicksort(&mut arr);
        let expected: Vec<i32> = (0..10_000).collect();
        assert_eq!(arr, expected);
    }
}
