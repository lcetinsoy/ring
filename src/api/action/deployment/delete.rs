use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse
};

use crate::api::server::Db;
use crate::models::deployments;

use crate::models::users::User;

pub(crate) async fn delete(Path(id): Path<String>, Extension(connexion): Extension<Db>, _user: User) -> impl IntoResponse {
    let guard = connexion.lock().await;
    let option = deployments::find(&guard, id);

    match option {
        Ok(Some(mut deployment)) => {
            deployment.status = "deleted".to_string();
            deployments::update(&guard, &deployment);

            StatusCode::NO_CONTENT
        }
        Ok(None) => {
            StatusCode::NOT_FOUND
        }

        Err(_) => {
            StatusCode::NO_CONTENT
        }
    }
}
