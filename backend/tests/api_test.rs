use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn test_app() -> axum::Router {
    let config = stellar_governance_voting_backend::state::Config {
        bind_addr: "0.0.0.0:0".parse().unwrap(),
        contract_id: String::new(),
        soroban_rpc_url: "http://localhost".into(),
        horizon_url: "http://localhost".into(),
        network_passphrase: "Test SDF Network ; September 2015".into(),
        network_name: "TESTNET".into(),
    };
    let state = stellar_governance_voting_backend::state::AppState::new(config);
    state.store.seed_demo_data();
    stellar_governance_voting_backend::app_router(state)
}

async fn body_str(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn health_ok() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"status\":\"ok\""));
}

#[tokio::test]
async fn status_has_counts() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/status").body(Body::empty()).unwrap()).await.unwrap();
    let body = body_str(res.into_body()).await;
    assert!(body.contains("\"proposal_count\":3"));
    assert!(body.contains("\"voter_count\":3"));
}

#[tokio::test]
async fn list_proposals() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/proposals").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_str(res.into_body()).await;
    assert!(body.contains("\"title\""));
}

#[tokio::test]
async fn get_proposal_by_id() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/proposals/0").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"id\":0"));
}

#[tokio::test]
async fn get_nonexistent_proposal_404() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/proposals/999").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_results() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/proposals/0/results").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_str(res.into_body()).await;
    assert!(body.contains("\"yes\""));
}

#[tokio::test]
async fn proposal_count() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/proposals/count").body(Body::empty()).unwrap()).await.unwrap();
    assert!(body_str(res.into_body()).await.contains("\"count\":3"));
}

#[tokio::test]
async fn voter_count() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/voters/count").body(Body::empty()).unwrap()).await.unwrap();
    assert!(body_str(res.into_body()).await.contains("\"count\":3"));
}

#[tokio::test]
async fn check_voter_eligible() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/voters/GA2C5RFPE6GCKMY3US5GAB29QZYE7DYXTZPGHLDJ5YBJO7VOZKAUBP2X").body(Body::empty()).unwrap()).await.unwrap();
    let body = body_str(res.into_body()).await;
    assert!(body.contains("\"is_eligible\":true"));
}

#[tokio::test]
async fn has_voted_check() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/voted/GA2C5RFPE6GCKMY3US5GAB29QZYE7DYXTZPGHLDJ5YBJO7VOZKAUBP2X/0").body(Body::empty()).unwrap()).await.unwrap();
    let body = body_str(res.into_body()).await;
    assert!(body.contains("\"has_voted\":true"));
}
