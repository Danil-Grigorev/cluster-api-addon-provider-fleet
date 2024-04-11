NAME := "cluster-api-fleet-controller"
KUBE_VERSION := env_var_or_default('KUBE_VERSION', '1.26.3')
ORG := "ghcr.io/rancher-sandbox"
TAG := "dev"

[private]
default:
    @just --list --unsorted --color=always

# run with opentelemetry
run-telemetry:
    OPENTELEMETRY_ENDPOINT_URL=http://127.0.0.1:55680 RUST_LOG=info,kube=trace,controller=debug cargo run --features=telemetry

# run without opentelemetry
run:
    RUST_LOG=info,kube=debug,controller=debug cargo run

# format with nightly rustfmt
fmt:
    cargo +nightly fmt

# run unit tests
test-unit:
  cargo test

# compile for musl (for docker image)
compile features="":
  #!/usr/bin/env bash
  docker run --rm \
    -v cargo-cache:/root/.cargo \
    -v $PWD:/volume \
    -w /volume \
    -t clux/muslrust:stable \
    cargo build --release --features={{features}} --bin controller
  cp target/x86_64-unknown-linux-musl/release/controller _out/controller

[private]
_build features="":
  just compile {{features}}
  docker build -t {{ORG}}/{{NAME}}:{{TAG}} .

# docker build base
build-base: (_build "")
# docker build with telemetry
build-otel: (_build "telemetry")

# Start local dev environment
start-dev:
    rm -rf _out/ || true
    kind delete cluster --name dev || true
    kind create cluster --config --image=kindest/node:v{{KUBE_VERSION}} --config testdata/kind-config.yaml
    just install-fleet
    just install-capi
    kubectl wait pods --for=condition=Ready --timeout=300s --all --all-namespaces

# Add and update helm repos used
update-helm-repos:
    helm repo add gitea-charts https://dl.gitea.com/charts/
    helm repo add fleet https://rancher.github.io/fleet-helm-charts/
    helm repo add jetstack https://charts.jetstack.io
    helm repo add traefik https://traefik.github.io/charts
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update

# Install fleet into the k8s cluster
install-fleet:
    #!/usr/bin/env bash
    set -euxo pipefail
    kubectl config view -o json --raw | jq -r '.clusters[].cluster["certificate-authority-data"]' | base64 -d > _out/ca.pem
    API_SERVER_URL=`kubectl config view -o json --raw | jq -r '.clusters[] | select(.name=="kind-dev").cluster["server"]'`
    helm -n cattle-fleet-system install --create-namespace --wait fleet-crd fleet/fleet-crd
    helm install --create-namespace -n cattle-fleet-system --set apiServerURL=$API_SERVER_URL --set-file apiServerCA=_out/ca.pem fleet fleet/fleet --wait

# Install cluster api and any providers
install-capi:
    clusterctl init -i docker

# Deploy will deploy the operator
deploy:
    kustomize build config/default | kubectl apply -f

undeploy:
    kustomize build config/default | kubectl delete --ignore-not-found=true -f -

release-manifests:
    kustomize build config/default > _out/addon-components.yaml

[private]
create-out-dir:
    mkdir -p _out

