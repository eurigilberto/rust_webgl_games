#[derive(Clone, Copy)]
pub struct Second(pub f64);
#[derive(Clone, Copy)]
pub struct Milisecond(pub f64);

impl From<Milisecond> for Second{
    fn from(time: Milisecond) -> Self {
        Second(time.0 / 1000.0)
    }
}
impl From<Second> for Milisecond{
    fn from(time: Second) -> Self {
        Milisecond(time.0 * 1000.0)
    }
}

impl std::ops::Mul<f32> for Second{
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        self.0 as f32 * rhs
    }
}

#[derive(Clone, Copy)]
pub struct FrameCount(pub u64);

pub struct Time {
    pub prev_time: Option<Milisecond>,
    pub time: Milisecond,
    pub frame_count: FrameCount,
    pub delta_time: Milisecond,
}
impl Time {
    pub fn new() -> Self {
        Self {
            prev_time: None,
            time: Milisecond(-1.0),
            frame_count: FrameCount(0),
            delta_time: Milisecond(0.0),
        }
    }

    pub fn update(&mut self, time: Milisecond) {
        match self.prev_time{
            Some(p_time) => {
                self.prev_time = Some(time);
                let delta_time = time.0 - p_time.0;
                self.delta_time = Milisecond(delta_time);
                self.time = Milisecond(self.time.0 + delta_time);
                self.frame_count.0 += 1;
            },
            None => {
                self.delta_time = Milisecond(0.0);
            },
        }
        self.prev_time = Some(time);
    }

    pub fn clear_time(&mut self){
        self.prev_time = None;
    }

    pub fn delta_time_seconds(&self) -> Second {
        self.delta_time.into()
    }
    pub fn time_seconds(&self) -> Second {
        self.time.into()
    }
}