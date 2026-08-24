use crate::memory::store::MemoryStore;
use anyhow::Result;

pub struct FeedbackManager;

impl FeedbackManager {
    pub fn record_feedback(
        store: &MemoryStore,
        id: &str,
        rating: u8,
        notes: &str,
    ) -> Result<()> {
        let summaries = store.list_summaries()?;
        if let Some(mut existing) = summaries.into_iter().find(|s| s.id == id) {
            existing.user_rating = Some(rating);
            existing.lessons_learned.push_str(&format!(" User Feedback: {}", notes));
            store.save_summary(&existing)?;
        }
        Ok(())
    }
}
