pub struct Sequence(u64);

impl Sequence {
    pub fn new() -> Sequence {
        Sequence(0)
    }

    pub fn pull_seq(&mut self) -> u64 {
        let seq = self.0;
        self.0 += 1;
        seq
    }
}
