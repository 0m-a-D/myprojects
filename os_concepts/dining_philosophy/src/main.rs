use dining_philosophy::Philosopher;
use std::{
    sync::{Arc, Mutex},
    thread,
};
fn main() {
    let p_config = Philosopher::create_config();
    let forks: Vec<Arc<Mutex<()>>> = (0..5).map(|_| Arc::new(Mutex::new(()))).collect();
    let handles: Vec<_> = p_config
        .into_iter()
        .map(|mut phil| {
            let forks = forks.clone();
            thread::spawn(move || {
                for _ in 0..3 {
                    phil.think();
                    phil.eat(&forks);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Dinner is over!");
}
