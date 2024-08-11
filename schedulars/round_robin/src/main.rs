// A simple round robin implementation in RUST for my kernel
use round_robin::schedular::RRScheduler;

fn main() {
    let mut scheduler = RRScheduler::new();
    let process_queue = RRScheduler::process_queue_generator();
    scheduler.execute_scheduler(process_queue);
}
