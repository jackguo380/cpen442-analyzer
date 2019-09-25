extern crate crc32fast;
extern crate fnv;
extern crate rand;

use std::thread;
use std::sync::mpsc;
use std::collections::hash_map::Entry;

use rand::Rng;
use rand::distributions::Standard;
use fnv::FnvHashMap;

const NCONS : usize = 16;
const NPROD_PER_CONS : usize = 4;
const CONS_CHUNK_SZ : usize = 256;

type CrcStr = (u32, String);

type CrcMap = FnvHashMap<u32, String>;

fn prod_thread(len: Option<usize>, tx_chan: mpsc::SyncSender<CrcStr>) {
    let mut rng = rand::thread_rng();

    loop {
        let mut h = crc32fast::Hasher::new();

        let s_len;
        if let Some(l) = len {
            s_len = rng.gen_range(10, std::cmp::max(11, l+4));
        } else {
            s_len = rng.gen_range(10, 20);
        }

        let s : String = rng.sample_iter(Standard)
            .filter(|b| (('0' as u8 <= *b) && (*b <= 'z' as u8)))
            .filter(|b| (*b != '\\' as u8))
            .map(|b| b as char)
            .take(s_len).collect();

        h.update(s.as_bytes());

        match tx_chan.send((h.finalize(), s)) {
            Ok(_) => continue,
            Err(_) => break
        }
    }

    println!("Producer thread exiting!");
}

fn cons_thread(
    check_collides : bool,
    tx_chan: mpsc::SyncSender<CrcMap>,
    rx_chan: mpsc::Receiver<CrcStr>) {

    loop {
        let mut map = CrcMap::default();

        for _ in 0..CONS_CHUNK_SZ {
            let s = rx_chan.recv().unwrap();

            match (check_collides, check_insert(&mut map, s)) {
                (true, Err(s)) => {
                    println!("Consumer Done: {}", s);
                    return;
                },
                (_, _) => continue
            }
        }

        match tx_chan.send(map) {
            Ok(_) => continue,
            Err(_) => break
        }
    }

    println!("Consumer thread exiting!");
}

fn find_thread(find_s: String, rx_chan: mpsc::Receiver<CrcMap>) {
    let find_crc = {
        let mut h = crc32fast::Hasher::new();
        h.update(find_s.as_bytes());
        h.finalize()
    };

    loop {
        let mut crcs = rx_chan.recv().unwrap();

        match check_insert(&mut crcs, (find_crc, find_s.clone())) {
            Ok(_) => continue,
            Err(s) => {
                println!("Join Done: {}", s);
                return;
            }
        }
    }
}

fn join_thread(rx_chan: mpsc::Receiver<CrcMap>) {
    let mut all_crcs = CrcMap::default();

    loop {
        let crcs = rx_chan.recv().unwrap();

        for s in crcs {
            match check_insert(&mut all_crcs, s) {
                Ok(_) => continue,
                Err(s) => {
                    println!("Join Done: {}", s);
                    return;
                }
            }
        }
    }
}

fn check_insert(map : &mut CrcMap, s : CrcStr) -> Result<(), String> {
    match (*map).entry(s.0) {
        Entry::Vacant(e) => {
            e.insert(s.1);
            Ok(())
        },
        Entry::Occupied(e) => {
            if *e.get() != s.1 {
                Err(format!("Collison ({:x}) '{}' and '{}'", s.0, e.get(), s.1))
            } else {
                Ok(())
            }
        }
    }
}

fn main() {
    let q = std::env::args().nth(1);

    let (tx_map, rx_map) = mpsc::sync_channel(16);

    let main_thread;
    let check_collides;
    let len;
    if let Some(q) = q {
        let mut h = crc32fast::Hasher::new();
        h.update(q.as_bytes());
        println!("CRC32 of {} = {}", q, h.finalize());

        check_collides = false;
        len = Some(q.len());

        main_thread = thread::spawn(move || { find_thread(q, rx_map); });
    } else {
        check_collides = true;
        len = None;

        main_thread = thread::spawn(move || { join_thread(rx_map); });
    }

    for _ in 0..NCONS {
        let tx_map = tx_map.clone();
        let (tx_str, rx_str) = mpsc::sync_channel(16);

        thread::spawn(move || { cons_thread(check_collides, tx_map, rx_str); });

        for _ in 0..NPROD_PER_CONS {
            let tx_str = tx_str.clone();

            thread::spawn(move || { prod_thread(len, tx_str); });
        }
    }

    main_thread.join().unwrap_or_else(|_| {
        println!("Could not join main thread");
    });
}
