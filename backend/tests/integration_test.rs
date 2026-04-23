//! Integration tests (requires test DB)
// cargo test --test integration_test

#[cfg(test)]
mod tests {
// use sqlx::postgres::PgPoolOptions; // Requires live DB
// use crate::modules::*; // Placeholder
    // Full e2e tests after docker

    #[allow(clippy::assertions_on_constants)]
    #[tokio::test]
    #[ignore]
    async fn test_full_flow() {
        // This is a full integration test requiring a live Postgres test DB.
        // If `TEST_DATABASE_URL` is not set the test will be skipped so local
        // `cargo test` doesn't fail. To run the full flow set up a test DB
        // and export `TEST_DATABASE_URL` then run `cargo test -- --ignored`.
        if std::env::var("TEST_DATABASE_URL").is_err() {
            eprintln!("Skipping integration test: TEST_DATABASE_URL not set");
            return;
        }

        // TODO: implement full flow using the test DB: create project, submit
        // an error, assert deduplication/count increments, etc.
    }

    #[test]
    fn test_smoke_non_db_helpers() {
        // Lightweight smoke test that exercises pure functions and helpers
        // when no test DB is available. This converts the previous placeholder
        // into a real, always-running test.
        use faultreport::modules::error_capture;

        let msg = "Smoke test message";
        let stack = "frame1\nframe2";
        let url = "https://example.com";

        let h = error_capture::compute_hash(msg, stack, url);
        assert!(!h.is_empty(), "hash should not be empty");

        // API key generator should produce a string with expected prefix
        let k = faultreport::api_key::generate_api_key();
        assert!(k.starts_with("frp_"));
    }
}

