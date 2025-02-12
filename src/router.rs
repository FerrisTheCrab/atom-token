use axum::routing::post;

pub struct InternalRouter;
pub struct Router;

impl Router {
    pub fn get() -> axum::Router {
        axum::Router::new()
            .route("/create", post(Router::create))
            .route("/find", post(Router::find))
            .route("/list", post(Router::list))
            .route("/remove", post(Router::remove))
            .route("/set", post(Router::set))
    }
}
