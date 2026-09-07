use rand::{Rng, RngExt};

pub trait VecRandomExt<T> {
    fn take_random<R: Rng>(&mut self, rng: &mut R) -> Option<T>;
}

impl<T> VecRandomExt<T> for Vec<T> {
    fn take_random<R: Rng>(&mut self, rng: &mut R) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let idx = rng.random_range(0..self.len());
        Some(self.swap_remove(idx))
    }
}
