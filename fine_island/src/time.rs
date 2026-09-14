pub fn clock(hours: usize, minutes: usize, seconds: usize) -> Clock {
    Clock {
        seconds: (hours * 3600) as u64 + (minutes * 60) as u64 + seconds as u64,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    seconds: u64,
}

impl Clock {
    pub fn minutes(self) -> usize {
        ((self.seconds / 60) % 60) as usize
    }
    pub fn hours(self) -> usize {
        (self.seconds / 3600) as usize
    }
}
