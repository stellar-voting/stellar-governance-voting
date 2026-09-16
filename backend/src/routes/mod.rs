pub mod health;
pub mod governance;

use axum::routing::{get, post};
use axum::Router;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/status", get(health::status))
        .route("/proposals", get(governance::list_proposals).post(governance::create_proposal))
        .route("/proposals/count", get(governance::proposal_count))
        .route("/proposals/:id", get(governance::get_proposal))
        .route("/proposals/:id/close", post(governance::close_proposal))
        .route("/proposals/:id/cancel", post(governance::cancel_proposal))
        .route("/proposals/:id/results", get(governance::get_results))
        .route("/vote", post(governance::cast_vote))
        .route("/voters/count", get(governance::voter_count))
        .route("/voters/register", post(governance::register_voter))
        .route("/voters/revoke", post(governance::revoke_voter))
        .route("/voters/:addr", get(governance::check_voter))
        .route("/voted/:addr/:pid", get(governance::has_voted))
        .route("/admin", get(governance::get_admin))
        .route("/pause", post(governance::pause))
        .route("/unpause", post(governance::unpause))
        .route("/paused", get(governance::is_paused));

    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", api)
        .with_state(state)
}
