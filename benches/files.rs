//! File area performance benchmarks
//!
//! This benchmark suite measures the performance of:
//! - File listing operations
//! - File search
//! - File counting and area retrieval

use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use impulse_file::{
    manager::InMemoryFileAreaManager,
    traits::FileAreaManager,
    types::{FileArea, SearchCriteria},
};
use impulse_types::{file::FileEntry, security::SecurityLevel};
use std::hint::black_box;

fn make_file(id: u32, area_id: u32, filename: &str, uploader: &str) -> FileEntry {
    FileEntry {
        id,
        filename: filename.to_string(),
        description: format!("Test file {}", id),
        uploader: uploader.to_string(),
        uploader_id: 1,
        size_bytes: 1024 * (id as u64 + 1),
        upload_date: Utc::now(),
        area_id,
        download_count: 0,
        is_offline: false,
        is_missing: false,
        password: None,
        cost_credits: None,
    }
}

fn benchmark_list_areas(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        for i in 0..10 {
            let area = FileArea::new(i, format!("Area {}", i), format!("Test area {}", i));
            manager.add_area(area).await.unwrap();
        }
        manager
    });

    c.bench_function("list_areas", |b| {
        b.iter(|| {
            rt.block_on(async { manager.list_areas(black_box(SecurityLevel::NEW_USER)).await })
        })
    });
}

fn benchmark_get_files(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea::new(1, "Test Area".to_string(), "Test".to_string());
        manager.add_area(area).await.unwrap();

        for i in 0..100 {
            let file = make_file(i, 1, &format!("file{}.zip", i), "Alice");
            manager.add_file(file).await.unwrap();
        }
        manager
    });

    c.bench_function("get_files_100", |b| {
        b.iter(|| rt.block_on(async { manager.get_files(black_box(1), 0, 100).await }))
    });
}

fn benchmark_get_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea::new(1, "Test Area".to_string(), "Test".to_string());
        manager.add_area(area).await.unwrap();

        let file = make_file(1, 1, "test.zip", "Alice");
        manager.add_file(file).await.unwrap();
        manager
    });

    c.bench_function("get_file", |b| {
        b.iter(|| rt.block_on(async { manager.get_file(black_box(1)).await }))
    });
}

fn benchmark_search_files(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea::new(1, "Test Area".to_string(), "Test".to_string());
        manager.add_area(area).await.unwrap();

        for i in 0..100 {
            let filename = if i % 2 == 0 {
                format!("file{}.zip", i)
            } else {
                format!("file{}.txt", i)
            };
            let uploader = if i % 3 == 0 { "Alice" } else { "Bob" };
            let file = make_file(i, 1, &filename, uploader);
            manager.add_file(file).await.unwrap();
        }
        manager
    });

    c.bench_function("search_files", |b| {
        b.iter(|| {
            rt.block_on(async {
                let criteria = SearchCriteria::new().with_filename(black_box("*.zip"));
                manager.search_files(&criteria).await
            })
        })
    });
}

fn benchmark_count_files(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea::new(1, "Test Area".to_string(), "Test".to_string());
        manager.add_area(area).await.unwrap();

        for i in 0..100 {
            let file = make_file(i, 1, &format!("file{}.zip", i), "Alice");
            manager.add_file(file).await.unwrap();
        }
        manager
    });

    c.bench_function("count_files", |b| {
        b.iter(|| rt.block_on(async { manager.count_files(black_box(1)).await }))
    });
}

fn benchmark_add_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("add_file", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = InMemoryFileAreaManager::new();
                let area = FileArea::new(1, "Test Area".to_string(), "Test".to_string());
                manager.add_area(area).await.unwrap();

                let file = make_file(1, 1, black_box("test.zip"), black_box("Alice"));
                manager.add_file(file).await
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_list_areas,
    benchmark_get_files,
    benchmark_get_file,
    benchmark_search_files,
    benchmark_count_files,
    benchmark_add_file,
);
criterion_main!(benches);
