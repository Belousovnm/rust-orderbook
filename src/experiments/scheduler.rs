use rand::Rng;

#[derive(Eq, PartialEq)]
pub enum Ready {
    Yes,
    No,
}

pub struct Schedule {
    pub counter: u32,
    pub cooldown: u32,
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            counter: 0,
            cooldown: rand::thread_rng().gen_range(10..20),
        }
    }

    pub const fn ready(&self) -> Ready {
        if self.counter >= self.cooldown {
            Ready::Yes
        } else {
            Ready::No
        }
    }

    pub fn set_counter(&mut self, counter: u32) {
        self.counter = counter;
    }

    pub fn incr_counter(&mut self) {
        self.counter += 1;
    }
}
