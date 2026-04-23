use actix_cors::Cors;
use actix_web::http::header;
use actix_web::web;
use crate::config::Config;
use crate::handlers;

pub fn config(config: &Config) -> impl Fn(&mut web::ServiceConfig) + Clone {
    let allowed = config.allowed_origins.clone();
    move |cfg: &mut web::ServiceConfig| {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        // Permit configured origins (supports comma-separated list)
        for origin in &allowed {
            cors = cors.allowed_origin(&format!("http://{}", origin));
            cors = cors.allowed_origin(&format!("https://{}", origin));
        }

        cfg.service(
            web::scope("/api")
                .wrap(cors)
                .route("/health", web::get().to(handlers::health))
                .route("/healthz", web::get().to(handlers::simple_health))
                .route("/projects", web::post().to(handlers::create_project))
                .route("/projects/{project_id}/errors", web::post().to(handlers::submit_error))
                .route("/projects/{project_id}/errors", web::get().to(handlers::list_errors))
        );
    }
}
