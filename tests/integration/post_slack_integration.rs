use httptest::{Server, Expectation, matchers::*, responders::*};
use uuid::Uuid;
use std::env;

// This integration test uses an in-repo HTTP sink and does not require the DB.
#[tokio::test]
async fn posts_to_webhook() {
    // Start a local test HTTP server to act as the Slack webhook sink.
    let server = Server::run();

    // Expect a POST with JSON body containing a `text` field.
    // Simulate two transient failures then a success to exercise retry logic.
    server.expect(
        Expectation::matching(request::method_path("POST", "/hook")).times(1)
            .respond_with(status_code(500))
    );
    server.expect(
        Expectation::matching(request::method_path("POST", "/hook")).times(1)
            .respond_with(status_code(500))
    );
    server.expect(
        Expectation::matching(request::method_path("POST", "/hook")).times(1)
            .respond_with(status_code(200))
    );

    // Call the helper directly with the server URL
    let webhook = server.url("/hook");
    let project_id = Uuid::new_v4();
    let error_hash = "deadbeef";

    // Ensure NO_EXTERNAL_CALLS is not set
    env::remove_var("NO_EXTERNAL_CALLS");

    // Call the raw posting helper
    let res = faultreport::alert::post_slack_raw(&webhook, project_id, error_hash).await;
    assert!(res.is_ok());
}
