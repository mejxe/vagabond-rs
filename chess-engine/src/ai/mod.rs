use std::time::Instant;

pub mod ai;
pub mod evaluation;

// TRAITS
pub trait TimeLimit {
    fn should_stop(&self) -> bool;
    fn restart(&mut self);
}
pub struct NoLimit;
impl TimeLimit for NoLimit {
    fn should_stop(&self) -> bool {
        false
    }
    fn restart(&mut self) {}
}
pub struct LimitedTime {
    pub start: Instant,
    pub allocated_time: u128,
}
impl TimeLimit for LimitedTime {
    fn should_stop(&self) -> bool {
        self.start.elapsed().as_millis() > self.allocated_time
    }
    fn restart(&mut self) {
        self.start = Instant::now();
    }
}
