use rand::Rng;

#[derive(Eq, PartialEq)]
pub enum Ready {
    Yes,
    No,
}

pub struct Schedule {
    pub counter: u64,
    pub cooldown: u64,
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new_rand()
    }
}

impl Schedule {
    pub fn new_rand() -> Self {
        Self {
            counter: 0,
            cooldown: rand::rng().random_range(10..20),
        }
    }
    pub const fn new(clock: u64) -> Self {
        Self {
            counter: 0,
            cooldown: clock,
        }
    }

    pub const fn ready(&self) -> Ready {
        if self.counter >= self.cooldown {
            Ready::Yes
        } else {
            Ready::No
        }
    }

    pub const fn set_counter(&mut self, counter: u64) {
        self.counter = counter;
    }

    pub const fn incr_counter(&mut self) {
        self.counter += 1;
    }
}
// TODO:
// next order update
// next ob update
