FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

ARG VERSION=v0.4.1
ARG TARGETARCH

RUN case "$TARGETARCH" in \
      amd64) \
        ARCH="x86_64-unknown-linux-gnu" && \
        EXPECTED="a2a9b1081a773bf3f0235b0cc4b97c274e7252f475977fc907609880f8235268" ;; \
      arm64) \
        ARCH="aarch64-unknown-linux-gnu" && \
        EXPECTED="64e8f3a643f76c03493827eb19ad6427771ff67bdea8a4e22b5819e857fc2fe1" ;; \
      *) echo "Unsupported arch: $TARGETARCH" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/kosakoytim/llm-wiki/releases/download/${VERSION}/${ARCH}.tar.gz" \
      -o /tmp/llm-wiki.tar.gz && \
    echo "${EXPECTED}  /tmp/llm-wiki.tar.gz" | sha256sum -c - && \
    tar -xzf /tmp/llm-wiki.tar.gz -C /usr/local/bin llm-wiki && \
    chmod +x /usr/local/bin/llm-wiki && \
    rm /tmp/llm-wiki.tar.gz

RUN useradd -m -u 1000 wiki
USER wiki

WORKDIR /wiki

# Wiki data directory — mount a persistent volume here
VOLUME ["/wiki/data"]

EXPOSE 8080

ENTRYPOINT ["llm-wiki"]
CMD ["serve", "--http", ":8080", "--wiki-path", "/wiki/data"]
