use rand::Rng;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::env::var;

fn main() {
    let read_threads: usize = var("READ_THREADS").unwrap().parse().unwrap();
    let duration_sec: u64 = var("DURATION_SEC").unwrap().parse().unwrap();
    let read_sleep_ms: u64 = var("READ_SLEEP_MS").unwrap().parse().unwrap();
    let write_interval_ms: u64 = var("WRITE_INTERVAL_MS").unwrap().parse().unwrap();
    let data_size: usize = var("DATA_SIZE").unwrap().parse().unwrap();

    // 初期データ
    let initial_data = Arc::new(vec![0u8; data_size]);
    // Arc<RwLock<Arc<T>>> という構造
    let shared = Arc::new(RwLock::new(initial_data));

    let read_threads = read_threads;
    let duration = Duration::from_secs(duration_sec);
    let start = Instant::now();

    let mut handles = vec![];

    // 読み取りスレッド
    // Arc<RwLock<Arc<T>>>の場合、読み取り時:
    // 1. read()ロックを取得
    // 2. guardからArc<T>をクローン（参照カウント増加のみ）
    // 3. guardをドロップしてロックを解放
    // 4. クローンしたArc<T>をロック無しでアクセス可能
    for _ in 0..read_threads {
        let s = shared.clone();
        handles.push(thread::spawn(move || {
            let mut read_count = 0;
            while start.elapsed() < duration {
                // 読み取りロック取得
                let arc_data = {
                    let guard = s.read().unwrap();
                    // guardはArc<T>への参照を持っているのでクローン
                    guard.clone()
                }; // ガードがドロップされるのでここでreadロック解放

                // ロック無しで重い処理を実行
                let _first_byte = arc_data[0];
                thread::sleep(Duration::from_millis(read_sleep_ms));

                read_count += 1;
            }
            read_count
        }));
    }

    // 書き込みスレッド（1つ）
    // Arc<RwLock<Arc<T>>>の場合、更新時:
    // write()ロック取得後、新しいArc<T>を生成して入れ替える
    let s = shared.clone();
    let writer_handle = thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let mut write_count = 0;
        while start.elapsed() < duration {
            thread::sleep(Duration::from_millis(write_interval_ms));
            let new_data = Arc::new(vec![rng.gen(); data_size]);
            let before = Instant::now();
            {
                let mut guard = s.write().unwrap();
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
