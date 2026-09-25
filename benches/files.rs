//! File area performance benchmarks
//!
//! This benchmark suite measures the performance of:
//! - File listing operations
//! - File search
//! - FILE_ID.DIZ extraction
//! - File area statistics

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use impulse_file::{
    manager::InMemoryFileAreaManager,
    traits::FileAreaManager,
    types::{FileArea, SearchCriteria},
};
use impulse_types::{
    file_entry::{FileEntry, FileName},
    security::SecurityLevel,
};
use std::path::PathBuf;

fn benchmark_list_areas(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create manager with 10 areas
    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        for i in 0..10 {
            let area = FileArea {
                id: i,
                name: format!("Area {}", i),
                description: format!("Test area {}", i),
                path: PathBuf::from(format!("/files/area{}", i)),
                security_level: SecurityLevel::NEW_USER,
                upload_security: SecurityLevel::REGULAR_USER,
            };
            manager.create_area(area).await.unwrap();
        }
        manager
    });

    c.bench_function("list_areas", |b| {
        b.iter(|| {
            rt.block_on(async {
                manager
                    .list_areas(black_box(SecurityLevel::NEW_USER))
                    .await
            })
        })
    });
}

fn benchmark_get_files(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create manager with area containing 100 files
    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea {
            id: 1,
            name: "Test Area".to_string(),
            description: "Test".to_string(),
            path: PathBuf::from("/files/test"),
            security_level: SecurityLevel::NEW_USER,
            upload_security: SecurityLevel::REGULAR_USER,
        };
        manager.create_area(area).await.unwrap();

        for i in 0..100 {
            let file = FileEntry::new(
                FileName::new(&format!("file{}.zip", i)).unwrap(),
                PathBuf::from("/files/test"),
            )
            .with_description(&format!("Test file {}", i))
            .with_uploader("Alice")
            .with_size(1024 * (i + 1));

            manager.add_file(1, file).await.unwrap();
        }
        manager
    });

    c.bench_function("get_files_100", |b| {
        b.iter(|| rt.block_on(async { manager.get_files(black_box(1), 0, 100).await }))
    });
}

fn benchmark_get_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create manager with one file
    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea {
            id: 1,
            name: "Test Area".to_string(),
            description: "Test".to_string(),
            path: PathBuf::from("/files/test"),
            security_level: SecurityLevel::NEW_USER,
            upload_security: SecurityLevel::REGULAR_USER,
        };
        manager.create_area(area).await.unwrap();

        let file = FileEntry::new(
            FileName::new("test.zip").unwrap(),
            PathBuf::from("/files/test"),
        )
        .with_description("Test file")
        .with_uploader("Alice")
        .with_size(1024);

        manager.add_file(1, file).await.unwrap();
        manager
    });

    c.bench_function("get_file", |b| {
        b.iter(|| rt.block_on(async { manager.get_file(black_box(1), black_box("test.zip")).await }))
    });
}

fn benchmark_search_files(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create manager with 100 files
    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea {
            id: 1,
            name: "Test Area".to_string(),
            description: "Test".to_string(),
            path: PathBuf::from("/files/test"),
            security_level: SecurityLevel::NEW_USER,
            upload_security: SecurityLevel::REGULAR_USER,
        };
        manager.create_area(area).await.unwrap();

        for i in 0..100 {
            let filename = if i % 2 == 0 {
                format!("file{}.zip", i)
            } else {
                format!("file{}.txt", i)
            };

            let file = FileEntry::new(FileName::new(&filename).unwrap(), PathBuf::from("/files/test"))
                .with_description(&format!("Test file {}", i))
                .with_uploader(if i % 3 == 0 { "Alice" } else { "Bob" })
                .with_size(1024 * (i + 1));

            manager.add_file(1, file).await.unwrap();
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

fn benchmark_area_stats(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create manager with 100 files
    let manager = rt.block_on(async {
        let mut manager = InMemoryFileAreaManager::new();
        let area = FileArea {
            id: 1,
            name: "Test Area".to_string(),
            description: "Test".to_string(),
            path: PathBuf::from("/files/test"),
            security_level: SecurityLevel::NEW_USER,
            upload_security: SecurityLevel::REGULAR_USER,
        };
        manager.create_area(area).await.unwrap();

        for i in 0..100 {
            let file = FileEntry::new(
                FileName::new(&format!("file{}.zip", i)).unwrap(),
                PathBuf::from("/files/test"),
            )
            .with_size(1024 * (i + 1));

            manager.add_file(1, file).await.unwrap();
        }
        manager
    });

    c.bench_function("area_stats", |b| {
        b.iter(|| rt.block_on(async { manager.get_area_stats(black_box(1)).await }))
    });
}

fn benchmark_add_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("add_file", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = InMemoryFileAreaManager::new();
                let area = FileArea {
                    id: 1,
                    name: "Test Area".to_string(),
                    description: "Test".to_string(),
                    path: PathBuf::from("/files/test"),
                    security_level: SecurityLevel::NEW_USER,
                    upload_security: SecurityLevel::REGULAR_USER,
                };
                manager.create_area(area).await.unwrap();

                let file = FileEntry::new(
                    FileName::new(black_box("test.zip")).unwrap(),
                    PathBuf::from("/files/test"),
                )
                .with_description(black_box("Test file"))
                .with_uploader(black_box("Alice"))
                .with_size(black_box(1024));

                manager.add_file(1, file).await
            })
        })
    });
}

fn benchmark_remove_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("remove_file", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = InMemoryFileAreaManager::new();
                let area = FileArea {
                    id: 1,
                    name: "Test Area".to_string(),
                    description: "Test".to_string(),
                    path: PathBuf::from("/files/test"),
                    security_level: SecurityLevel::NEW_USER,
                    upload_security: SecurityLevel::REGULAR_USER,
                };
                manager.create_area(area).await.unwrap();

                let file = FileEntry::new(
                    FileName::new("test.zip").unwrap(),
                    PathBuf::from("/files/test"),
                );
                manager.add_file(1, file).await.unwrap();

                manager.remove_file(black_box(1), black_box("test.zip")).await
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
    benchmark_area_stats,
    benchmark_add_file,
    benchmark_remove_file
);
criterion_main!(benches);
