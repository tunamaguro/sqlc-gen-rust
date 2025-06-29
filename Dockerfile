FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN apt-get update -y && \
    apt-get install -y \
    git \
    curl \
    wget \
    unzip

# Install protoc
RUN curl https://github.com/protocolbuffers/protobuf/releases/download/v29.3/protoc-29.3-linux-x86_64.zip -Lo protoc.zip
RUN unzip -q protoc.zip bin/protoc 'include/*' -d /usr/local && rm protoc.zip

# Copy sqlc bin
COPY --from=sqlc/sqlc:1.28.0 /workspace/sqlc /usr/bin/sqlc

ARG USERNAME=vscode
USER ${USERNAME}

# Add completions
RUN echo "source /usr/share/bash-completion/completions/git" >> /home/${USERNAME}/.bashrc
RUN echo "source <( rustup completions bash )" >> /home/${USERNAME}/.bashrc
RUN echo "source <( rustup completions bash cargo )" >> /home/${USERNAME}/.bashrc

RUN rustup component add rustfmt clippy
RUN cargo install just