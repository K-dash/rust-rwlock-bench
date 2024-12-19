use std::env;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::Arc;
use parking_lot::RwLock;
use rand::Rng;

fn main() {
    // 環境変数からパラメータ取得
    let read_threads: usize = env::var("READ_THREADS").unwrap().parse().unwrap();
    let duration_sec: u64 = env::var("DURATION_SEC").unwrap().parse().unwrap();
    let read_sleep_ms: u64 = env::var("READ_SLEEP_MS").unwrap().parse().unwrap();
    let write_interval_ms: u64 = env::var("WRITE_INTERVAL_MS").unwrap().parse().unwrap();
    let data_size: usize = env::var("DATA_SIZE").unwrap().parse().unwrap();

    let duration = Duration::from_secs(duration_sec);

    // 初期データ
    let initial_data = vec![0u8; data_size];
    let shared = Arc::new(RwLock::new(initial_data));

    let start = Instant::now();
    let mut handles = vec![];

    // 読み取りスレッド
    for _ in 0..read_threads {
        let s = shared.clone();
        handles.push(thread::spawn(move || {
            let mut read_count = 0;
            while start.elapsed() < duration {
                {
                    let guard = s.read();
                    let _ = guard[0];
                    // 重い処理をロック保持中に行う
                    // （Arc<RwLock<T>>同様、ロック保持中の処理）
                    thread::sleep(Duration::from_millis(read_sleep_ms));
                }
                read_count += 1;
            }
            read_count
        }));
    }

    // 書き込みスレッド
    let s = shared.clone();
    let writer_handle = thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let mut write_count = 0;
        while start.elapsed() < duration {
            thread::sleep(Duration::from_millis(write_interval_ms));
            let new_data = vec![rng.gen(); data_size];
            let before = Instant::now();
            {
                let mut guard = s.write();
                *guard = new_data;
            }
            let latency = before.elapsed();
            println!("Write done, latency: {:?}", latency);
            write_count += 1;
        }
        write_count
    });
    handles.push(writer_handle);

    // 結果集計
    let mut total_reads = 0;
    let mut total_writes = 0;
    for (i, h) in handles.into_iter().enumerate() {
        let res = h.join().unwrap();
        if i == read_threads {
            // 最後に追加したスレッドが書き込みスレッド
            total_writes = res;
        } else {
            total_reads += res;
        }
    }

    println!("Total reads: {}", total_reads);
    println!("Total writes: {}", total_writes);
}
