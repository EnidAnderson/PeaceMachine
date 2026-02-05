#[derive(Debug, Clone, Copy)]
pub struct Fuel(pub u64);

impl Fuel {
    pub fn consume(&mut self, n: u64) -> bool {
        if self.0 >= n {
            self.0 -= n;
            true
        } else {
            false
        }
    }
}
