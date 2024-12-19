use arc_swap::ArcSwap;
use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::env::var;

fn main() {
    let read_threads: usize = var("READ_THREADS").unwrap().parse().unwrap();
    let duration_sec: u64 = var("DURATION_SEC").unwrap().parse().unwrap();
    let read_sleep_ms: u64 = var("READ_SLEEP_MS").unwrap().parse().unwrap();
    let write_interval_ms: u64 = var("WRITE_INTERVAL_MS").unwrap().parse().unwrap();
    let data_size: usize = var("DATA_SIZE").unwrap().parse().unwrap();

    let read_threads = read_threads;
    let duration = Duration::from_secs(duration_sec);
    let start = Instant::now();

    // 初期データ
    let initial_data = Arc::new(vec![0u8; data_size]);
    let shared = Arc::new(ArcSwap::new(initial_data));

    let mut handles = vec![];

    // 読み取りスレッド
    // arc_swapは読み取り時にロック不要で即座にArc<T>を取得できる
    // クローン操作はArcカウント増減のみで軽量
    // ロックなしでHeavyな処理を行える
    for _ in 0..read_threads {
        let s = shared.clone();
        handles.push(thread::spawn(move || {
            let mut read_count = 0;
            while start.elapsed() < duration {
                // ロック不要でArc<T>を取得
                // arc_swapはロックがないので、明示的なロック解除用スコープは不要
                let arc_data = s.load();
                let _first_byte = arc_data[0];
                // ロックなしで重い処理を実行
                thread::sleep(Duration::from_millis(read_sleep_ms));

                read_count += 1;
            }
            read_count
        }));
    }

    // 書き込みスレッド
    // arc_swapではstoreで原子的に参照先を差し替えるだけ
    // 読み取り中のスレッドは古いArc<T>を保持し続けられるので安全
    let s = shared.clone();
    let writer_handle = thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let mut write_count = 0;
        while start.elapsed() < duration {
            thread::sleep(Duration::from_millis(write_interval_ms));
            let new_data = Arc::new(vec![rng.gen(); data_size]);
            let before = Instant::now();
            s.store(new_data);
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
            total_writes = res;
        } else {
            total_reads += res;
        }
    }

    println!("Total reads: {}", total_reads);
    println!("Total writes: {}", total_writes);
}
