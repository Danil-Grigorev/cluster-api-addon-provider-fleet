FROM --platform=${BUILDPLATFORM} ghcr.io/rust-cross/rust-musl-cross:aarch64-musl AS build-arm64
ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y clang musl-tools
RUN find / -name libc-header-start.h; exit 1
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --target aarch64-unknown-linux-musl  --default-toolchain stable
RUN curl -LO https://go.dev/dl/go1.24.0.linux-amd64.tar.gz \
 && rm -rf /usr/local/go && tar -C /usr/local -xzf go1.24.0.linux-amd64.tar.gz \
 && rm go1.24.0.linux-amd64.tar.gz

ENV PATH=/usr/local/go/bin:/root/.cargo/bin:$PATH

WORKDIR /usr/src

RUN mkdir /usr/src/controller
WORKDIR /usr/src/controller
COPY ./ ./

ARG features=""
ENV CC=musl-gcc
RUN RUST_BACKTRACE=1 cargo install --locked --target aarch64-unknown-linux-musl --features=${features} --path .

FROM --platform=${BUILDPLATFORM} ghcr.io/rust-cross/rust-musl-cross:x86_64-musl AS build-amd64
ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y clang musl-tools
RUN find / -name libc-header-start.h; exit 1
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --target x86_64-unknown-linux-musl  --default-toolchain stable
RUN curl -LO https://go.dev/dl/go1.24.0.linux-amd64.tar.gz \
 && rm -rf /usr/local/go && tar -C /usr/local -xzf go1.24.0.linux-amd64.tar.gz \
 && rm go1.24.0.linux-amd64.tar.gz

ENV PATH=/usr/local/go/bin:/root/.cargo/bin:$PATH

WORKDIR /usr/src

RUN mkdir /usr/src/controller
WORKDIR /usr/src/controller
COPY ./ ./

ARG features=""
ENV CC=musl-gcc
RUN RUST_BACKTRACE=1 cargo install --locked --target x86_64-unknown-linux-musl --features=${features} --path .

FROM --platform=amd64 registry.suse.com/suse/helm:3.17 AS helm-amd64
FROM --platform=arm64 registry.suse.com/suse/helm:3.17 AS helm-arm64

FROM helm-amd64 AS target-amd64
COPY --from=build-amd64 --chmod=0755 /root/.cargo/bin/controller /apps/controller

FROM helm-arm64 AS target-arm64
COPY --from=build-arm64 --chmod=0755 /root/.cargo/bin/controller /apps/controller

FROM target-${TARGETARCH}
ENV PATH="${PATH}:/apps"
EXPOSE 8080
ENTRYPOINT ["/apps/controller"]