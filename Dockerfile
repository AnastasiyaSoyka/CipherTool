FROM public.ecr.aws/ubuntu/ubuntu:24.04 as BUILDER

ARG DEBIAN_FRONTEND=noninteractive

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt update && \
    apt install --no-install-recommends --yes \
    build-essential curl tree ca-certificates

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR /build

COPY . .

RUN /root/.cargo/bin/cargo build --release && tree -a .

FROM public.ecr.aws/ubuntu/ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive

LABEL maintainer="apsoyka@protonmail.com"

RUN printf 'CREATE_MAIL_SPOOL=no' >> /etc/default/useradd && \
    mkdir -p /home/ciphertool /home/scripts && \
    groupadd ciphertool && \
    useradd ciphertool -g ciphertool -d /home/ciphertool && \
    chown ciphertool:ciphertool /home/ciphertool

COPY --from=BUILDER --chown=ciphertool:ciphertool /build/target/release/ciphertool /usr/bin/ciphertool

USER ciphertool:ciphertool
WORKDIR /home/ciphertool
ENTRYPOINT [ "/usr/bin/ciphertool" ]
CMD [ "--help" ]
