use crate::{process::Process, process_state::ProcessState};
use rand::{thread_rng, Rng};

const TIME_QUANTUM: u8 = 2; // milliseconds

pub struct RRScheduler {
    ready_queue: Vec<Process>,
    completed_processes: Vec<Process>,
    current_time: u8,
}
impl RRScheduler {
    pub fn new() -> RRScheduler {
        Self {
            ready_queue: Vec::new(),
            completed_processes: Vec::new(),
            current_time: 0,
        }
    }

    pub fn process_queue_generator() -> Vec<Process> {
        let mut process_queue = Vec::with_capacity(5);
        let first_process = Process {
            pid: Some(thread_rng().gen::<u8>()),
            p_burst_time: thread_rng().gen_range(1..8),
            p_arrival_time: 0,
            p_state: ProcessState::Ready,
            p_remaining_time: 0,
        };
        process_queue.push(first_process);
        process_queue[0].p_remaining_time = process_queue[0].p_burst_time;

        for _ in 1..process_queue.capacity() {
            process_queue.push(Process::new());
        }
        process_queue.sort_by(|p1, p2| p1.p_arrival_time.cmp(&p2.p_arrival_time));

        for (index, process) in process_queue.iter().enumerate() {
            println!(
                "Process number: {} with process PID: {:?} has burst time: {} and arrival time: {}",
                index + 1,
                process.pid.unwrap(),
                process.p_burst_time,
                process.p_arrival_time,
            );
        }
        println!();
        process_queue
    }

    pub fn execute_scheduler(&mut self, mut process_queue: Vec<Process>) {
        while !process_queue.is_empty() || !self.ready_queue.is_empty() {
            // Move arrived processes to the ready queue
            process_queue.retain(|process| {
                if process.p_arrival_time <= self.current_time {
                    self.ready_queue.push(*process);
                    false
                } else {
                    true
                }
            });

            if let Some(mut current_process) = self.ready_queue.first().cloned() {
                self.ready_queue.remove(0);
                let execution_time = current_process.p_remaining_time.min(TIME_QUANTUM);
                self.current_time += execution_time;
                current_process.p_remaining_time -= execution_time;

                if current_process.p_remaining_time == 0 {
                    current_process.p_state = ProcessState::Complete;
                    println!(
                        "-> Process with id: {} completed at time {}!",
                        current_process.pid.unwrap(),
                        self.current_time
                    );
                    println!();
                    self.completed_processes.push(current_process);
                } else {
                    current_process.p_state = ProcessState::Ready;
                    self.ready_queue.push(current_process);
                }
            } else if !process_queue.is_empty() {
                // If no process is ready, go to the next arrival
                self.current_time = process_queue[0].p_arrival_time;
            }

            println!(
                "Time: {}, Ready Queue: {:?}",
                self.current_time, self.ready_queue
            );
            println!();
        }

        self.print_statistics();
    }

    fn print_statistics(&self) {
        let total_turnaround_time: u8 = self
            .completed_processes
            .iter()
            .map(|p| self.current_time - p.p_arrival_time)
            .sum();
        let avg_turnaround_time =
            total_turnaround_time as f32 / self.completed_processes.len() as f32;

        let total_waiting_time: u8 = self
            .completed_processes
            .iter()
            .map(|p| self.current_time - p.p_arrival_time - p.p_burst_time)
            .sum();
        let avg_waiting_time = total_waiting_time as f32 / self.completed_processes.len() as f32;

        println!("Average Turnaround Time: {:.2}", avg_turnaround_time);
        println!("Average Waiting Time: {:.2}", avg_waiting_time);
    }
}

impl Default for RRScheduler {
    fn default() -> Self {
        Self::new()
    }
}
