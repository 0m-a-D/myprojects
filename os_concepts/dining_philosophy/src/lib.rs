use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
const EATING_TIME: u64 = 2;
const THINKING_TIME: u64 = 1;

#[derive(Debug)]
pub enum Action {
    Thinking,
    Hungry,
    Eating,
}

#[derive(Debug)]
pub struct Philosopher {
    id: usize,
    action: Action,
    left_fork: usize,
    right_fork: usize,
}
impl Philosopher {
    pub fn create_config() -> Vec<Philosopher> {
        let mut config = Vec::with_capacity(5);
        let p0 = Philosopher {
            id: 0,
            action: Action::Thinking,
            left_fork: 4,
            right_fork: 0,
        };
        config.push(p0);
        for i in 1..5 {
            let philosopher = Philosopher {
                id: i,
                action: Action::Thinking,
                left_fork: i - 1,
                right_fork: i,
            };
            config.push(philosopher);
        }
        config
    }
    pub fn eat(&mut self, forks: &[Arc<Mutex<()>>]) {
        let left = &forks[self.left_fork];
        let right = &forks[self.right_fork];

        let _left_lock = left.lock().unwrap();
        let _right_lock = right.lock().unwrap();
        self.action = Action::Eating;
        println!("Philosopher {} is Eating", self.id);
        thread::sleep(Duration::from_secs(EATING_TIME));
    }
    pub fn think(&mut self) {
        self.action = Action::Thinking;
        println!("Philosopher {} is Thinking", self.id);
        thread::sleep(Duration::from_secs(THINKING_TIME));
    }
}
