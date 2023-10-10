FROM rust:alpine3.18 as builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/latex2img_rs
COPY . .
RUN cargo install --path .

FROM alpine:3.18
RUN apk add --no-cache texlive texmf-dist-most imagemagick
COPY --from=builder /usr/local/cargo/bin/latex2img_rs /usr/local/bin/latex2img_rs

CMD ["latex2img_rs"]