//! Message system performance benchmarks
//!
//! This benchmark suite measures the performance of:
//! - JAM message base operations
//! - Message parsing and formatting
//! - Message list rendering
//! - Search operations
//! - Thread building

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use impulse_message::{
    formats::JamMessageBase,
    traits::MessageBase,
    types::{NewMessage, SearchCriteria},
};
use std::path::PathBuf;

fn benchmark_message_post(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_post", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = tempfile::tempdir().unwrap();
                let base_path = temp_dir.path().join("bench_base");

                let mut base = JamMessageBase::new(&base_path);
                base.create().await.unwrap();

                let msg = NewMessage::new(
                    black_box("Alice"),
                    black_box("Bob"),
                    black_box("Test Subject"),
                )
                .with_body(black_box("This is a test message body."));

                base.post_message(msg).await
            })
        })
    });
}

fn benchmark_message_read(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create base with one message
    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let mut base = JamMessageBase::new(&base_path);
        base.create().await.unwrap();

        let msg = NewMessage::new("Alice", "Bob", "Test").with_body("Body");
        base.post_message(msg).await.unwrap();

        (base_path.clone(), temp_dir)
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

    // Setup: create base with 100 messages
    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let mut base = JamMessageBase::new(&base_path);
        base.create().await.unwrap();

        for i in 0..100 {
            let msg = NewMessage::new("Alice", "Bob", format!("Message {}", i));
            base.post_message(msg).await.unwrap();
        }

        (base_path.clone(), temp_dir)
    });

    c.bench_function("message_list_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let base = JamMessageBase::new(&base_path);
                base.list_messages(black_box(0), black_box(100)).await
            })
        })
    });
}

fn benchmark_message_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create base with 100 messages
    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let mut base = JamMessageBase::new(&base_path);
        base.create().await.unwrap();

        for i in 0..100 {
            let from = if i % 2 == 0 { "Alice" } else { "Bob" };
            let msg = NewMessage::new(from, "All", format!("Message {}", i));
            base.post_message(msg).await.unwrap();
        }

        (base_path.clone(), temp_dir)
    });

    c.bench_function("message_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let base = JamMessageBase::new(&base_path);
                let criteria = SearchCriteria::new().with_from(black_box("Alice"));
                base.search(&criteria).await
            })
        })
    });
}

fn benchmark_thread_building(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create base with threaded messages
    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let mut base = JamMessageBase::new(&base_path);
        base.create().await.unwrap();

        // Create a thread with 10 messages
        let msg1 = NewMessage::new("Alice", "All", "Thread Topic");
        base.post_message(msg1).await.unwrap();

        for i in 0..9 {
            let original = base.read_message(1).await.unwrap();
            let reply = impulse_message::reply::ReplyBuilder::new(original)
                .quote_original(false)
                .build("User", format!("Reply {}", i));
            base.post_message(reply).await.unwrap();
        }

        (base_path.clone(), temp_dir)
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

    // Setup: create base with 100 messages
    let (base_path, _temp_dir) = rt.block_on(async {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("bench_base");

        let mut base = JamMessageBase::new(&base_path);
        base.create().await.unwrap();

        for i in 0..100 {
            let msg = NewMessage::new("Alice", "Bob", format!("Message {}", i));
            base.post_message(msg).await.unwrap();
        }

        (base_path.clone(), temp_dir)
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
