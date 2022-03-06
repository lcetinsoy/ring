use shiplift::{ContainerOptions, Docker, PullOptions, NetworkCreateOptions, ContainerConnectionOptions};
use futures::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use crate::models::deployments::Deployment;

#[tokio::main]
pub(crate) async fn apply(mut config: Deployment) {
    let docker = Docker::new();

    info!("docker runtime search");

    match docker.containers().list(&Default::default()).await {
        Ok(containers) => {
            for container in containers {
                let container_id = &container.id;

                for (label, value) in container.labels.into_iter() {
                    if "ring_deployment" == label && value == config.id {
                        config.instances.push(container_id.to_string());
                    }
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }


    if config.status == "delete" {
        println!("delete container {:?}", config.instances);
        for instance in config.instances.iter_mut() {
            println!("{}", instance);

            remove_container(docker.clone(), instance.to_string()).await;

            info!("container {} delete", instance);
        }
    } else {
        let number_instances = config.instances.len();
        let n_us: i64 = (number_instances as i16).into();
        println!("{:?}", n_us);

        if n_us < config.replicas {
            info!("create container {}", config.image.clone());

            create_container(&mut config, &docker).await
        }

        if n_us > config.replicas {
            let first_container_id = &config.instances[0];

            remove_container(docker.clone(), first_container_id.to_string()).await;
        }

        debug!("docker runtime apply {:?}", config);
    }

}

async fn pull_image(docker: Docker, image: String) {

    println!("pull docker image: {}", image);
    info!("pull docker image: {}", image);

    let mut stream = docker
        .images()
        .pull(&PullOptions::builder().image(image).build());

     while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(_output) => { }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

async fn create_container(config: &mut Deployment, docker: &Docker) {

    // @todo: Add a check to see if the image is already pulled
    pull_image(docker.clone(), config.image.to_string()).await;

    let network_name = format!("ring_{}", config.namespace.clone());
    create_network(docker.clone(), network_name.clone()).await;

    let mut container_options = ContainerOptions::builder(config.image.as_str());
    let mut labels = HashMap::new();

    labels.insert("ring_deployment", config.id.as_str());

    let labels_format = Deployment::deserialize_labels(&config.labels);

    for (key, value) in labels_format.iter() {
        labels.insert(key, value);
    }

    container_options.labels(&labels);

    match docker
        .containers()
        .create(&container_options.build())
        .await
    {
        Ok(container) => {
            debug!("{:?}", container.id);
            config.instances.push(container.id.to_string());

            let networks = docker.networks();
            networks
                .get(&network_name)
                .connect(&ContainerConnectionOptions::builder(&container.id).build())
                .await;

            docker.containers().get(container.id).start().await;
        },
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn remove_container(docker: Docker, container_id: String) {
    match docker.containers().get(&container_id).stop(Some(Duration::from_millis(10))).await {
        Ok(_info) => {
            println!("{:?}", _info);
        },
        Err(_e) => {
            println!("{:?}", _e);
        },
    };

    info!("remove container: {}", &container_id);
}

async fn create_network(docker: Docker, network_name: String) {

    debug!("create network: {}", network_name);

    match docker.networks().get(&network_name).inspect().await {
        Ok(_network_info) => {
            println!("{:?}", _network_info);
            debug!("network {:?} already exist", network_name);
        },
        Err(e) => {
            info!("create network: {}", network_name);

            match docker
                .networks()
                .create(
                    &NetworkCreateOptions::builder(network_name.as_ref())
                        .driver("bridge")
                        .build(),
                )
                .await
            {
                Ok(info) => println!("{:?}", info),
                Err(_e) => eprintln!("Error: {}", e),
            }
        },
    }
}