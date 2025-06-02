use criterion::{Criterion, criterion_group, criterion_main};
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio_fusion::{BatchExecutor, ThreadPoolBuilder, ThreadPoolResult};

async fn cpu_intensive_task(iterations: u64) -> ThreadPoolResult<u64> {
    let mut result: u64 = 0;
    for i in 0..iterations {
        result = result.wrapping_add(i);
    }
    Ok(result)
}

async fn io_intensive_task() -> ThreadPoolResult<()> {
    // Simulate I/O by sleeping
    tokio::time::sleep(Duration::from_micros(100)).await;
    Ok(())
}

fn bench_cpu_intensive(b: &mut criterion::Bencher, task_count: usize) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    let pool = rt.block_on(async {
        Arc::new(
            ThreadPoolBuilder::new()
                .worker_threads(num_cpus::get())
                .build(),
        )
    });

    b.iter(|| {
        rt.block_on(async {
            let mut batch = BatchExecutor::new(Arc::clone(&pool));
            for _ in 0..task_count {
                batch.add_task(cpu_intensive_task(1000), 1);
            }
            let results = batch.execute().await;
            for result in results {
                result.unwrap();
            }
        });
    });
}

fn bench_tokio_spawn_cpu(b: &mut criterion::Bencher, task_count: usize) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    b.iter(|| {
        rt.block_on(async {
            let handles: Vec<_> = (0..task_count)
                .map(|_| tokio::spawn(cpu_intensive_task(1000)))
                .collect();

            let results = join_all(handles).await;
            for result in results {
                result.unwrap().unwrap();
            }
        });
    });
}

fn bench_io_intensive(b: &mut criterion::Bencher, task_count: usize) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    let pool = rt.block_on(async {
        Arc::new(
            ThreadPoolBuilder::new()
                .worker_threads(num_cpus::get())
                .build(),
        )
    });

    b.iter(|| {
        rt.block_on(async {
            let mut batch = BatchExecutor::new(Arc::clone(&pool));
            for _ in 0..task_count {
                batch.add_task(io_intensive_task(), 1);
            }
            let results = batch.execute().await;
            for result in results {
                result.unwrap();
            }
        });
    });
}

fn bench_tokio_spawn_io(b: &mut criterion::Bencher, task_count: usize) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    b.iter(|| {
        rt.block_on(async {
            let handles: Vec<_> = (0..task_count)
                .map(|_| tokio::spawn(io_intensive_task()))
                .collect();

            let results = join_all(handles).await;
            for result in results {
                result.unwrap().unwrap();
            }
        });
    });
}

fn bench_thread_pool(c: &mut Criterion) {
    // Test with different task counts
    for &task_count in &[10, 100, 1000] {
        // CPU-bound benchmarks
        let mut group = c.benchmark_group(format!("CPU Intensive ({} tasks)", task_count));

        // Benchmark for tokio-fusion (CPU-bound)
        group.bench_function("tokio-fusion", |b| {
            bench_cpu_intensive(b, task_count);
        });

        // Benchmark for tokio::spawn (CPU-bound)
        group.bench_function("tokio::spawn", |b| {
            bench_tokio_spawn_cpu(b, task_count);
        });

        group.finish();

        // I/O-bound benchmarks
        let mut group = c.benchmark_group(format!("I/O Intensive ({} tasks)", task_count));

        // Benchmark for tokio-fusion (I/O-bound)
        group.bench_function("tokio-fusion", |b| {
            bench_io_intensive(b, task_count);
        });

        // Benchmark for tokio::spawn (I/O-bound)
        group.bench_function("tokio::spawn", |b| {
            bench_tokio_spawn_io(b, task_count);
        });

        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(10));
    targets = bench_thread_pool
}

criterion_main!(benches);
