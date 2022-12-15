FROM lukemathwalker/cargo-chef:latest-rust-1.56.0 AS chef
WORKDIR /gotify_sample

###########
## Planner
###########
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

###########
## Builder
###########
FROM chef AS builder

# Create appuser
ENV USER=myuser
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN apt-get install --no-install-recommends --assume-yes protobuf-compiler
RUN update-ca-certificates

# Copy recipe and build dependencies
COPY --from=planner /gotify_sample/recipe.json recipe.json
RUN cargo chef cook --release  --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Build the actual program
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

###############
## Final image
###############
FROM scratch

# Import user from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /gotify_sample
# Copy our build
COPY --from=builder /gotify_sample/target/x86_64-unknown-linux-musl/release/gotify-news-reader-server ./

USER myuser:myuser
ENV RUST_LOG=info
CMD ["/gotify_sample/gotify-news-reader-server"]