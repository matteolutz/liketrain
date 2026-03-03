pub struct Throttle<F> {
    millis_fn: F,
    min_iteration_millis: u32,

    last_iteration: u32,
}

impl<F> Throttle<F>
where
    F: Fn() -> u32,
{
    pub fn new(millis_fn: F, min_iteration_millis: u32) -> Self {
        let now = millis_fn();
        Self {
            millis_fn,
            min_iteration_millis,
            last_iteration: now,
        }
    }

    fn millis(&self) -> u32 {
        (self.millis_fn)()
    }

    pub fn throttle(&mut self) {
        let now = self.millis();
        let took = now - self.last_iteration;

        if took >= self.min_iteration_millis {
            self.last_iteration = now;
            return;
        }

        arduino_hal::delay_ms(self.min_iteration_millis - took);
        self.last_iteration = self.millis();
    }
}
