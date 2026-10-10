use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::application::ReviewCompletedEvent;
use crate::domain::card::Card;
use crate::domain::errors::JabeXError;
use crate::domain::review::{Grade, Review};
use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitReviewInput {
    pub card_id: Uuid,
    pub grade: Grade,
    pub duration_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOutcomeDto {
    pub card: Card,
    pub review: Review,
}

#[tauri::command]
pub async fn list_due_cards(
    state: State<'_, AppState>,
    deck_id: Option<Uuid>,
    limit: Option<u32>,
) -> Result<Vec<Card>, JabeXError> {
    tracing::debug!(command = "list_due_cards", "command invoked");
    state.with_session(|session| session.reviews.list_due_window(deck_id, None, limit))
}

#[tauri::command]
pub async fn submit_review(
    state: State<'_, AppState>,
    input: SubmitReviewInput,
) -> Result<ReviewOutcomeDto, JabeXError> {
    tracing::debug!(command = "submit_review", card_id = %input.card_id, "command invoked");
    state.with_session(|session| {
        let outcome =
            session
                .reviews
                .submit_review(input.card_id, input.grade, input.duration_ms, None)?;

        // Project each graded review into daily study activity (streaks / heatmap).
        let event = ReviewCompletedEvent::from_review(
            session.profile_id,
            outcome.card.deck_id,
            outcome.review.duration_ms,
        );
        if let Err(err) = session.study_activity.handle_review_completed(&event) {
            tracing::warn!(
                error = %err,
                card_id = %input.card_id,
                "failed to record study activity"
            );
        }

        Ok(ReviewOutcomeDto {
            card: outcome.card,
            review: outcome.review,
        })
    })
}
