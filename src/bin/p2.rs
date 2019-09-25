extern crate analyzer;
extern crate rayon;
extern crate rand;
extern crate cipher_crypt;
extern crate ctrlc;
extern crate thread_priority;

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};
use thread_priority::*;
use std::default::Default;
use std::f64;

use rand::Rng;
use rand::seq::SliceRandom;
use cipher_crypt::{Cipher, Playfair};

use analyzer::score::{NgramScore4, NgramScore2, WordListScore};
use analyzer::simann::*;

const NCPU : usize = 32;
const PLAYFAIR_ALPHABET : &str = "ABCDEFGHIKLMNOPQRSTUVWXYZ";
const MAX_FAIL : u64 = 800;
const MAX_RESULTS : usize = 16;
const PRINT_SECS : u64 = 20;
const HIGH_COVERAGE : f32 = 0.75;
const WORDLIST_FACTOR : f64 = 0.25;
const TEMP: i64 = 20;

fn generate_key<T: Rng>(rng : &mut T) -> String {
    //let r : f64 = rng.gen();
    //if false {
        let mut k = Vec::from(PLAYFAIR_ALPHABET);
        k.shuffle(rng);

        String::from_utf8(k).unwrap()
    //} else {
    //    String::from("EBODVGZCXUKYQHFNWSPTRMIAL")
    //}
}

fn random_swap_key<T: Rng>(key : &str, rng : &mut T) -> String {
    let r : f64 = rng.gen();

    if r > 0.95 {
        // Reverse the Key
        key.chars().rev().collect()
    } else if r > 0.90 {
        // Swap Columns
        let mut key = String::from(key);
        let cols = rand::seq::index::sample(rng, 5, 2).into_vec();

        unsafe {
            let key = key.as_bytes_mut();

            for i in 0..5 {
                let swp = key[i * 5 + cols[0]];
                key[i * 5 + cols[0]] = key[i * 5 + cols[1]];
                key[i * 5 + cols[1]] = swp;
            }
        }

        key
    } else if r > 0.85 {
        // Swap Rows
        let mut key = String::from(key);
        let rows = rand::seq::index::sample(rng, 5, 2).into_vec();

        unsafe {
            let key = key.as_bytes_mut();

            for i in 0..5 {
                let swp = key[i + rows[0] * 5];
                key[i + rows[0] * 5] = key[i + rows[1] * 5];
                key[i + rows[1] * 5] = swp;
            }
        }

        key
    } else {
        let mut key = String::from(key);
        let idxs = rand::seq::index::sample(rng, key.len(), 2).into_vec();

        unsafe {
            let key = key.as_bytes_mut();

            let c1 = key[idxs[0]];
            let c2 = key[idxs[1]];

            key[idxs[0]] = c2;
            key[idxs[1]] = c1;
        }

        key
    }
}

fn score_text(
    ng_score: &NgramScore4,
    ng2_score: &NgramScore2,
    wl_score: &WordListScore,
    text : &str) -> f64 {

    let score = ng_score.score(&text);
    //let score2 = ng2_score.score(&cur_decrypt);
    let word_coverage = wl_score.coverage(&text);

    let score = score * 0.5 + 0.5 * score * (1.0 - word_coverage) as f64;
    //let score = score + score2;

    score
}

fn simulated_annealing<DFn>(cipher : &str,
    ng_score: &NgramScore4,
    ng2_score: &NgramScore2,
    wl_score: &WordListScore,
    decrypt_fn: DFn) -> SimulatedAnnResult 
    where DFn : Fn(&str, &str) -> String {
    let mut rng = rand::thread_rng();

    let mut last_key = generate_key(&mut rng);
    let mut last_decrypt = decrypt_fn(cipher, &last_key);
    let mut last_score = score_text(ng_score, ng2_score, wl_score, &last_decrypt);
    let mut best_key = last_key.clone();
    let mut best_decrypt = last_decrypt.clone();
    let mut best_score = last_score;

    for temp in 0..TEMP {
        let mut fail_count = 0;

        while fail_count < MAX_FAIL {
            let cur_key = random_swap_key(&last_key, &mut rng);

            let cur_decrypt = decrypt_fn(cipher, &cur_key);

            let cur_score = score_text(ng_score, ng2_score, wl_score, &cur_decrypt);
            if cur_score > last_score {
                last_key = cur_key;
                last_decrypt = cur_decrypt;
                last_score = cur_score;
            } else {
                let pow = (cur_score - last_score) / (TEMP-temp) as f64;
                let prob = pow.exp();

                if prob > rng.gen() {
                    last_key = cur_key;
                    last_decrypt = cur_decrypt;
                    last_score = cur_score;
                }
            }

            if last_score > best_score {
                best_key = last_key.clone();
                best_decrypt = last_decrypt.clone();
                best_score = last_score;
                fail_count = 0;
            } else {
                fail_count += 1;
            }
        }
    }

    let word_coverage = wl_score.coverage(&best_decrypt);

    SimulatedAnnResult {
        key: best_key,
        decrypt: best_decrypt,
        score: best_score,
        word_coverage
    }
}

fn pf_decrypt(cipher: &str, key: &str) -> String {
    let pf = Playfair::new((String::from(key), None));
    pf.decrypt(cipher).unwrap()
}

fn join_thread_run(rx_chan: mpsc::Receiver<SimulatedAnnResult>,
    run: &Mutex<bool>) {

    set_thread_priority(thread_native_id(), ThreadPriority::Max,
        ThreadSchedulePolicy::Normal(NormalThreadSchedulePolicy::Normal)).unwrap();

    let mut best_heap = Vec::<SimulatedAnnResult>::new();
    //let mut best_result : Option<SimulatedAnnResult> = None;
    let mut last_result_counter = 0;
    let mut result_counter = 0;
    let mut print_time = SystemTime::now() + Duration::from_secs(PRINT_SECS);

    loop {
        if *run.lock().unwrap() == false {
            break;
        }

        let res = rx_chan.recv().unwrap();

        result_counter += 1;

        if res.word_coverage > HIGH_COVERAGE {
            println!("High Word Coverage: {}", res);
        }

        //if is_finished(&res.decrypt) {
        //    best_result = Some(res);
        //    break;
        //}

        handle_annealing_result(&mut best_heap, res);

        if print_time < SystemTime::now() {
            print_time = SystemTime::now() + Duration::from_secs(PRINT_SECS);

            print_results(&best_heap);

            println!("Current Rate {} Results/s", (result_counter - last_result_counter) as f32 / PRINT_SECS as f32);

            last_result_counter = result_counter;
        }
    }

    print_results(&best_heap);

    //if let Some(best_result) = best_result {
    //    println!("Found result: {}", best_result);
    //}
}

//fn is_finished(decrypt : &str) -> bool {
//    decrypt.find("COMMA").is_some()
//        && decrypt.find("DOT").is_some()
//}

fn handle_annealing_result(all_results: &mut Vec<SimulatedAnnResult>,
    new_res: SimulatedAnnResult) {

    match all_results.binary_search_by(|a| new_res.cmp(a)) {
        Ok(idx) => all_results[idx] = new_res,
        Err(idx) => all_results.insert(idx, new_res)
    };

    if all_results.len() > MAX_RESULTS {
        all_results.resize(MAX_RESULTS, Default::default());
    }
}

fn print_results(all_results: &Vec<SimulatedAnnResult>) {
    println!("Periodic Results: ");

    for i in all_results.iter().enumerate() {
        println!("{}: {}\n", i.0, i.1);
    }

    println!("End Results");
}

fn main() {
    let mut file = File::open("cipher2.txt").expect("Cannot open cipher2.txt");

    let ngram_score = NgramScore4::create("english_quadgrams.txt");

    let ngram2_score = NgramScore2::create("english_bigrams.txt");

    assert!(ngram_score.total < ngram2_score.total);

    let wl_score = WordListScore::create("wordlist.txt");

    let mut cipher = Vec::new();

    file.read_to_end(&mut cipher).unwrap();

    let cipher = String::from_utf8(cipher).unwrap();
    let cipher : String = cipher.chars().filter(|c| !c.is_whitespace()).collect();
    let cipher = cipher.to_uppercase();
    assert_eq!(cipher.chars().filter(|c| *c < 'A' || 'Z' < *c).count(), 0);

    println!("Cipher. Score = {} Cipher = {}",
        ngram_score.score(&cipher), cipher);

    let cipher = Arc::new(cipher);
    let ngram_score = Arc::new(ngram_score);
    let ngram2_score = Arc::new(ngram2_score);
    let wl_score = Arc::new(wl_score);

    let (tx_chan, rx_chan) = mpsc::sync_channel(NCPU * 4);

    let run = Arc::new(Mutex::new(true));

    let handler_run = run.clone();

    ctrlc::set_handler(move || {
        let mut run = handler_run.lock().unwrap();
        if *run {
            println!("Signal received! Preparing to stop...");
            *run = false;
        } else {
            println!("Signal received again! Dying...");
            std::process::abort();
        }
    }).expect("Failed to install signal handler");


    let join_run = run.clone();
    let join_thread = thread::spawn(move || {
        join_thread_run(rx_chan, &join_run);
    });


    let mut worker_threads = Vec::new();

    for _ in 0..NCPU {
        let tx_chan = tx_chan.clone();
        let cipher = cipher.clone();
        let ngram_score = ngram_score.clone();
        let ngram2_score = ngram2_score.clone();
        let wl_score = wl_score.clone();

        worker_threads.push(thread::spawn(move || {
            loop {
                let res = simulated_annealing(
                    &cipher,
                    &ngram_score,
                    &ngram2_score,
                    &wl_score,
                    pf_decrypt);

                match tx_chan.send(res) {
                    Ok(_) => continue,
                    Err(_) => {
                        println!("Worker thread exiting!");
                        return;
                    }
                }
            }
        }));
    }

    for t in worker_threads {
        t.join().unwrap_or_else(|_| println!("Failed to join thread"));
    }

    join_thread.join().unwrap_or_else(|_| println!("Join thread failed to join"));
}
