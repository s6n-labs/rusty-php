FROM debian:bookworm-slim AS php

WORKDIR /usr/local/src/php

RUN apt-get update && apt-get install -qq autoconf bison gcc git make re2c
RUN git clone --depth 1 --single-branch -b PHP-8.2.7 https://github.com/php/php-src.git .

RUN ./buildconf && \
    ./configure \
        --disable-all \
        --disable-cgi \
        --enable-embed \
        --enable-zts \
        --enable-zend-max-execution-timers \
        --disable-zend-signals && \
    make -j "$(nproc)" && \
    make install

RUN php --version

FROM rustlang/rust:nightly-bookworm-slim AS rust

WORKDIR /usr/local/src/rusty-php

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./build/Cargo.toml ./build/
COPY ./cli/Cargo.toml ./cli/
COPY ./core/Cargo.toml ./core/
COPY ./http/Cargo.toml ./http/
COPY ./sys/Cargo.toml ./sys/
RUN mkdir -p ./build/src ./cli/src ./core/src ./http/src ./sys/src && \
    echo 'fn main() {}' | tee ./cli/src/main.rs | tee ./http/src/main.rs && \
    touch ./build/src/lib.rs ./core/src/lib.rs ./sys/src/lib.rs && \
    cargo build --release

COPY --from=php /lib/ /lib/
COPY --from=php /usr/local/bin/php /usr/local/bin/php
COPY --from=php /usr/local/include/php/ /usr/local/include/php
COPY --from=php /usr/local/lib/libphp.* /usr/local/lib
COPY --from=php /usr/local/lib/php/ /usr/local/lib/php
COPY --from=php /usr/local/php/ /usr/local/php

COPY . .
RUN cargo build --release

FROM php

COPY --from=rust /usr/local/src/rusty-php/target/release/rusty-php-cli /usr/bin/rusty-php

ENTRYPOINT ["/usr/bin/rusty-php"]
