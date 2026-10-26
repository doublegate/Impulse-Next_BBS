//! Message system performance benchmarks
//!
//! This benchmark suite measures the performance of:
//! - JAM message base operations
//! - Message parsing and formatting
//! - Message list rendering
//! - Search operations
//! - Thread building

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use impulse_message::{
    formats::{JamMessageBase, jam::JamWriter},
    traits::MessageBase,
    types::{NewMessage, SearchCriteria},
};
use std::hint::black_box;

fn benchmark_message_post(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_post", |b| {
        b.iter_batched_ref(
            || {
                rt.block_on(async {
                    let temp_dir = tempfile::tempdir().unwrap();
                    let base_path = temp_dir.path().join("bench_base");

                    let writer = JamWriter::new(&base_path);
                    writer.initialize_base().await.unwrap();

                    (temp_dir, JamMessageBase::new(&base_path))
                })
            },
            |(_, base)| {
                rt.block_on(async {
                    let msg = NewMessage::new(
                        black_box("Alice"),
                        black_box("Bob"),
                        black_box("Test Subject"),
                    )
                    .with_body(black_box("This is a test message body."));

                    base.post_message(msg).await
                })
            },
            BatchSize::PerIteration,
        )
    });
}

fn benchmark_message_read(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let writer = JamWriter::new(&base_path);
        writer.initialize_base().await.unwrap();
        let mut base = JamMessageBase::new(&base_path);

        let msg = NewMessage::new("Alice", "Bob", "Test").with_body("Body");
        base.post_message(msg).await.unwrap();

        (base_path, temp_dir)
    });

    c.bench_function("message_read", |b| {
        b.iter(|| {
            rt.block_on(async {
                let base = JamMessageBase::new(&base_path);
                base.read_message(black_box(1)).await
            })
        })
    });
}

fn benchmark_message_list(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let writer = JamWriter::new(&base_path);
        writer.initialize_base().await.unwrap();
        let mut base = JamMessageBase::new(&base_path);

        for i in 0..100 {
            let msg =
                NewMessage::new("Alice", "Bob", format!("Message {}", i)).with_body("Message body");
            base.post_message(msg).await.unwrap();
        }

        (base_path, temp_dir)
    });

    c.bench_function("message_list_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let base = JamMessageBase::new(&base_path);
                base.list_messages(black_box(1), black_box(100)).await
            })
        })
    });
}

fn benchmark_message_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let writer = JamWriter::new(&base_path);
        writer.initialize_base().await.unwrap();
        let mut base = JamMessageBase::new(&base_path);

        for i in 0..100 {
            let from = if i % 2 == 0 { "Alice" } else { "Bob" };
            let msg =
                NewMessage::new(from, "All", format!("Message {}", i)).with_body("Message body");
            base.post_message(msg).await.unwrap();
        }

        let initial_matches = base
            .search(&SearchCriteria::new().with_from("Alice"))
            .await
            .unwrap();
        assert_eq!(initial_matches.len(), 50);

        (base_path, temp_dir)
    });

    c.bench_function("message_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let base = JamMessageBase::new(&base_path);
                let criteria = SearchCriteria::new().with_from(black_box("Alice"));
                let matches = base.search(&criteria).await.unwrap();
                assert_eq!(matches.len(), 50);
                matches
            })
        })
    });
}

fn benchmark_thread_building(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let writer = JamWriter::new(&base_path);
        writer.initialize_base().await.unwrap();
        let mut base = JamMessageBase::new(&base_path);

        let msg1 = NewMessage::new("Alice", "All", "Thread Topic").with_body("Root post");
        base.post_message(msg1).await.unwrap();

        for i in 0..9 {
            let original = base.read_message(1).await.unwrap();
            let reply = impulse_message::reply::ReplyBuilder::new(original)
                .quote_original(false)
                .build("User", format!("Reply {}", i));
            base.post_message(reply).await.unwrap();
        }

        (base_path, temp_dir)
    });

    c.bench_function("thread_build", |b| {
        b.iter(|| {
            rt.block_on(async {
                let base = JamMessageBase::new(&base_path);
                base.get_thread(black_box(1)).await
            })
        })
    });
}

fn benchmark_message_count(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let writer = JamWriter::new(&base_path);
        writer.initialize_base().await.unwrap();
        let mut base = JamMessageBase::new(&base_path);

        for i in 0..100 {
            let msg =
                NewMessage::new("Alice", "Bob", format!("Message {}", i)).with_body("Message body");
            base.post_message(msg).await.unwrap();
        }

        (base_path, temp_dir)
    });

    c.bench_function("message_count", |b| {
        b.iter(|| {
            rt.block_on(async {
                let base = JamMessageBase::new(&base_path);
                base.message_count().await
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_message_post,
    benchmark_message_read,
    benchmark_message_list,
    benchmark_message_search,
    benchmark_thread_building,
    benchmark_message_count
);
criterion_main!(benches);
