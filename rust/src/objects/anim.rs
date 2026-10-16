//! Frame counter shared by animated objects (objects/anim.cc).

#[derive(Clone, Copy, Default, Debug)]
pub struct Anim {
    current_delay: i32,
    current_frame: i32,
}

impl Anim {
    /// Advances every `num_repeats + 1` calls; wraps after `num_frames` (inclusive).
    pub fn next_frame(&mut self, num_frames: i32, num_repeats: i32) -> i32 {
        self.current_delay += 1;
        if self.current_delay > num_repeats {
            self.current_delay = 0;
            self.current_frame += 1;
        }
        if self.current_frame > num_frames {
            self.current_frame = 0;
        }
        self.current_frame
    }

    pub fn reset(&mut self) -> i32 {
        self.current_delay = 0;
        self.current_frame = 0;
        0
    }

    pub fn set_frame(&mut self, f: i32) {
        self.current_frame = f;
    }
}
