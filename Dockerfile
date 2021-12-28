####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN apt-get -y upgrade

RUN update-ca-certificates

# Create appuser
ENV USER=dataregi
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /dataregi

COPY .cargo .
COPY Cargo.lock .
COPY Cargo.toml .
COPY dummy.rs .

RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY ./src ./src
COPY ./migrations ./migrations
COPY ./templates ./templates

RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM debian:buster-slim

RUN apt-get -y update
RUN apt-get -y install openssl postgresql-client ca-certificates

RUN update-ca-certificates

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /dataregi

# Copy our build
COPY --from=builder /dataregi/target/release/dataregi ./

# Use an unprivileged user.
USER dataregi:dataregi

COPY Rocket.toml .
COPY ./dataregi.com ./dataregi.com
COPY ./migrations ./migrations
COPY ./site ./site
COPY ./templates ./templates

CMD ["/dataregi/dataregi"]