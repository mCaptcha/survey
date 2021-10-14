FROM node:14.16.0 as frontend
LABEL org.opencontainers.image.source https://github.com/mCaptcha/survey
RUN apt-get update && apt-get install -y make
COPY package.json yarn.lock /src/
COPY vendor/ /src/vendor
WORKDIR /src
RUN yarn install
COPY . .
RUN make frontend

FROM rust:1.51.0-slim-buster as rust
WORKDIR /src
RUN apt-get update && apt-get install -y git
COPY . /src
COPY --from=frontend /src/static/cache/bundle /src/static/cache/bundle
ENV RUST_BACKTRACE=1
RUN cargo build --release

FROM debian:bullseye-slim
RUN useradd -ms /bin/bash -u 1001 mcaptcha-survey
WORKDIR /home/mcaptcha-survey
COPY --from=rust /src/target/release/survey /usr/local/bin/
COPY --from=rust /src/config/default.toml /etc/mcaptcha-survey/config.toml
USER mcaptcha-survey
CMD [ "/usr/local/bin/survey" ]
