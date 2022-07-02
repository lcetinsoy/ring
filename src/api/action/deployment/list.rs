use axum::{
    extract::{Extension},
    response::IntoResponse,
    Json,
};

use crate::api::server::hydrate_deployment_output;
use crate::api::server::Db;
use crate::api::dto::deployment::DeploymentOutput;
use crate::models::deployments;
use crate::runtime::docker;

pub(crate) async fn list(Extension(connexion): Extension<Db>) -> impl IntoResponse {

    let mut deployments: Vec<DeploymentOutput> = Vec::new();
    let guard = connexion.lock().await;

    let list_deployments = deployments::find_all(guard);

    for deployment in list_deployments.into_iter() {
        let d = deployment.clone();

        let mut output = hydrate_deployment_output(deployment);
        let instances = docker::list_instances(d.id.to_string()).await;
        output.instances = instances;

        deployments.push(output);
    }

    Json(deployments)
}