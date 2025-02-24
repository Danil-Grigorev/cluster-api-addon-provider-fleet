FROM --platform=${BUILDPLATFORM} ghcr.io/cross-rs/aarch64-unknown-linux-musl:0.2.5 AS build-arm64
ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --target aarch64-unknown-linux-musl  --default-toolchain stable

ENV PATH=/root/.cargo/bin:$PATH
RUN cargo --version

WORKDIR /usr/src

RUN mkdir /usr/src/controller
WORKDIR /usr/src/controller
COPY ./ ./

ARG features=""
RUN cargo install --locked --target aarch64-unknown-linux-musl --features=${features} --path .

FROM --platform=${BUILDPLATFORM} ghcr.io/cross-rs/x86_64-unknown-linux-musl:0.2.5 AS build-amd64
ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --target x86_64-unknown-linux-musl  --default-toolchain stable

ENV PATH=/root/.cargo/bin:$PATH
RUN cargo --version

WORKDIR /usr/src

RUN mkdir /usr/src/controller
WORKDIR /usr/src/controller
COPY ./ ./

ARG features=""
RUN cargo install --locked --target x86_64-unknown-linux-musl --features=${features} --path .

FROM --platform=amd64 registry.suse.com/suse/helm:3.13 AS helm-amd64
FROM --platform=arm64 registry.suse.com/suse/helm:3.13 AS helm-arm64

FROM scratch AS copy-amd64
COPY --from=build-amd64 --chmod=0755 /root/.cargo/bin/controller /apps/controller
COPY --from=helm-amd64 --chmod=0755 /usr/bin/helm /apps/helm

FROM scratch AS copy-arm64
COPY --from=build-arm64 --chmod=0755 /root/.cargo/bin/controller /apps/controller
COPY --from=helm-arm64 --chmod=0755 /usr/bin/helm /apps/helm

# Copy the statically-linked binary into a scratch container.
FROM copy-${TARGETARCH}
COPY ./Cargo.lock /Cargo.lock
ENV PATH="${PATH}:/apps"
EXPOSE 8080
ENTRYPOINT ["/apps/controller"]
