use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Render {
    // ID
    pub user_id: String,
    pub id: String,

    // File
    pub file_id: String,
    pub file_version: i32,

    // Range
    pub frame_start: i32,
    pub frame_end: i32,
    pub step: i32,
    pub slices: i32,

    // Pointer
    pub pointer_frame: i32,
    pub pointer_slice: i32,

    // Completion status
    pub total_jobs: i32,
    pub completed_jobs: i32,

    // Billing
    pub subscription_item_id: String,
}

impl Render {
    pub fn new(
        user_id: String,
        id: String,
        file_id: String,
        file_version: i32,
        frame_start: i32,
        frame_end: i32,
        step: i32,
        slices: i32,
        subscription_item_id: String,
    ) -> Self {
        Self {
            user_id,
            id,
            file_id,
            file_version,
            frame_start,
            frame_end,
            step,
            slices,
            pointer_frame: frame_start,
            pointer_slice: 0,
            total_jobs: Self::total_jobs(frame_start, frame_end, step, slices),
            completed_jobs: 0,
            subscription_item_id,
        }
    }

    pub fn total_jobs(frame_start: i32, frame_end: i32, step: i32, slices: i32) -> i32 {
        let frames = 1 + (frame_end - frame_start) / step;

        frames * slices
    }

    pub fn get_job(&self, worker_id: String) -> Option<Job> {
        if self.is_queue_drained() {
            return None;
        }

        Some(Job {
            user_id: self.user_id.clone(),
            render_id: self.id.clone(),
            frame: self.pointer_frame,
            slice: self.pointer_slice,

            file_id: self.file_id.clone(),
            file_version: self.file_version,
            total_slices: self.slices,
            worker_id,
        })
    }

    // Returns true if the render is complete
    pub fn advance_pointer(&mut self) -> &mut Self {
        self.pointer_slice += 1;
        if self.pointer_slice >= self.slices {
            self.pointer_slice = 0;
            self.pointer_frame += self.step;
        }

        self
    }

    pub fn is_queue_drained(&self) -> bool {
        self.pointer_frame > self.frame_end
    }

    pub fn is_complete(&self) -> bool {
        self.completed_jobs >= self.total_jobs
    }

    pub fn is_first(&self) -> bool {
        self.pointer_frame == self.frame_start && self.pointer_slice == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    // ID
    pub user_id: String,
    pub render_id: String,
    pub frame: i32,
    pub slice: i32,

    // Metadata
    pub file_id: String,
    pub file_version: i32,
    pub total_slices: i32,
    pub worker_id: String,
}
