#!/bin/sh
set -e

WIKI_CONFIG="/wiki/config.toml"
WIKI_RUNTIME_CONFIG="/tmp/wiki-runtime.toml"
SCHEMAS_SRC="/wiki/schemas"

# User-configurable env vars
WIKI_NAME="${WIKI_NAME:-main}"
WIKI_DESCRIPTION="${WIKI_DESCRIPTION:-Personal AI memory wiki}"
WIKI_GIT_EMAIL="${WIKI_GIT_EMAIL:-wiki@llm-wiki}"
WIKI_GIT_NAME="${WIKI_GIT_NAME:-llm-wiki}"
# WIKI_ALLOWED_HOSTS: comma-separated list of allowed hosts
# e.g. WIKI_ALLOWED_HOSTS=myapp.example.com,localhost
WIKI_ALLOWED_HOSTS="${WIKI_ALLOWED_HOSTS:-localhost}"
WIKI_PORT="${WIKI_PORT:-8080}"

WIKI_DATA="/wiki/data/${WIKI_NAME}"

# Fix volume ownership if running as root (happens when Docker mounts override image ownership)
if [ "$(id -u)" = "0" ]; then
    chown -R wiki:wiki /wiki/data
    exec gosu wiki "$0" "$@"
fi

# Build runtime config by appending allowed_hosts from env to base config
cp "$WIKI_CONFIG" "$WIKI_RUNTIME_CONFIG"

# Generate http_allowed_hosts TOML array from comma-separated env var
HOSTS_TOML=$(echo "$WIKI_ALLOWED_HOSTS" | tr ',' '\n' | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//' | awk '{ printf "    \"%s\",\n", $0 }')
cat >> "$WIKI_RUNTIME_CONFIG" << EOF

[serve]
http = true
http_port = ${WIKI_PORT}
http_allowed_hosts = [
    "localhost",
    "127.0.0.1",
$HOSTS_TOML]
EOF

# Initialize wiki space on first boot
if [ ! -f "$WIKI_DATA/wiki.toml" ]; then
    echo "Initializing wiki space at $WIKI_DATA..."

    mkdir -p "$WIKI_DATA"

    llm-wiki --config "$WIKI_RUNTIME_CONFIG" spaces create \
        "$WIKI_DATA" \
        --name "$WIKI_NAME" \
        --description "$WIKI_DESCRIPTION" \
        --set-default || { echo "ERROR: spaces create failed"; exit 1; }

    mkdir -p "$WIKI_DATA/schemas"
    for SCHEMA in identity relationship preference routine project context daily_summary event episode task_context lesson; do
        cp "$SCHEMAS_SRC/${SCHEMA}.json" "$WIKI_DATA/schemas/${SCHEMA}.json"
        echo "Copied schema: $SCHEMA"
    done

    cd "$WIKI_DATA" && \
        git config user.email "$WIKI_GIT_EMAIL" && \
        git config user.name "$WIKI_GIT_NAME" && \
        git add schemas/ && \
        git commit -m "chore: register 11 custom brain-modeled schemas" && \
        cd /wiki

    echo "Wiki initialized with custom schemas."
fi

exec llm-wiki --config "$WIKI_RUNTIME_CONFIG" serve --http ":${WIKI_PORT}"
