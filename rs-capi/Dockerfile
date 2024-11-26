FROM rust:1.81.0 AS builder

RUN printf "\
    [source.crates-io] \n\
    replace-with = 'rsproxy-sparse'\n\
    \n\
    [source.rsproxy-sparse]\n\
    registry = 'sparse+https://rsproxy.cn/index/'\n\
    " > $CARGO_HOME/config.toml
# RUN cargo install crm && crm use rsproxy-sparse

WORKDIR /workspace
ADD . /workspace
RUN cargo build --bin rs-capi --release



FROM ubuntu:22.04
# RUN apt-get update && apt-get install -y --no-install-recommends \
#     ca-certificates \
#     && rm -rf /var/lib/apt/lists/*

COPY --from=builder /workspace/target/release/rs-capi /workspace/rs-capi

WORKDIR /workspace
CMD ["./rs-capi"]