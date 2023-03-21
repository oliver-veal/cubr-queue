use super::entity::Render;
use rand::seq::SliceRandom;

pub async fn select_render(renders: Vec<Render>) -> Option<Render> {
    renders
        .into_iter()
        .filter(|r| !r.is_queue_drained())
        .collect::<Vec<Render>>()
        .choose(&mut rand::thread_rng())
        .cloned()
}
