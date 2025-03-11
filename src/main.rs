use actix_web::{
    get, middleware, post,
    web::{self, post, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
pub use controller::{self, telemetry, State};
use prometheus::{Encoder, TextEncoder};
use serde_json::json;

#[get("/metrics")]
async fn metrics(c: Data<State>, _req: HttpRequest) -> impl Responder {
    let metrics = c.metrics();
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&metrics, &mut buffer).unwrap();
    HttpResponse::Ok().body(buffer)
}

#[get("/health")]
async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[get("/")]
async fn index(c: Data<State>, _req: HttpRequest) -> impl Responder {
    let d = c.diagnostics().await;
    HttpResponse::Ok().json(&d)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init().await;

    // Init k8s controller state
    let state = State::new();

    match state.flags.helm_install {
        true => {
            let helm_install_controller = controller::run_fleet_helm_controller(state.clone());
            tokio::join!(helm_install_controller);
        }
        false => {
            let fleet_config_controller =
                controller::run_fleet_addon_config_controller(state.clone());
            let cluster_controller = controller::run_cluster_controller(state.clone());
            let cluster_class_controller = controller::run_cluster_class_controller(state.clone());

            // Start web server
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(Data::new(state.clone()))
                    .wrap(middleware::Logger::default().exclude("/health"))
                    .service(index)
                    .service(health)
                    .service(metrics)
                    .service(validate_topology)
                    .service(generate_patches)
                    .service(discover_variables)
            })
            .bind("0.0.0.0:8443")?
            .shutdown_timeout(5)
            .run();

            tokio::join!(
                cluster_controller,
                cluster_class_controller,
                fleet_config_controller,
                server
            )
            .3?;
        }
    };
    Ok(())
}

#[post("/hooks.runtime.cluster.x-k8s.io/v1alpha1/generatepatches/generate-patches")]
async fn generate_patches() -> impl Responder {
    web::Json(json!({}))
}

#[post("/hooks.runtime.cluster.x-k8s.io/v1alpha1/validatetopology/validate-topology")]
async fn validate_topology() -> impl Responder {
    web::Json(json!({}))
}

#[post("/hooks.runtime.cluster.x-k8s.io/v1alpha1/discovery/discover-variables")]
async fn discover_variables() -> impl Responder {
    web::Json(json!({}))
}
