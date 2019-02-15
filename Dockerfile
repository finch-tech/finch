FROM rust:1.32

WORKDIR /usr/src/finch
COPY . .

RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install --path .

EXPOSE 8000

CMD ["finch"]
