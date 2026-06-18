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
else
    # On redeploy the runtime config is fresh but the volume already has wiki data.
    # Re-register the existing space so the engine can mount it.
    echo "Re-registering existing wiki space '$WIKI_NAME' at $WIKI_DATA..."
    llm-wiki --config "$WIKI_RUNTIME_CONFIG" spaces register \
        "$WIKI_DATA" \
        --name "$WIKI_NAME" \
        --description "$WIKI_DESCRIPTION" || { echo "ERROR: spaces register failed"; exit 1; }
    echo "Wiki space '$WIKI_NAME' re-registered."
fi

# --- System space (AI behavior rules) ---
SYSTEM_DATA="/wiki/data/system"

if [ ! -f "$SYSTEM_DATA/wiki.toml" ]; then
    echo "Initializing system wiki space at $SYSTEM_DATA..."

    mkdir -p "$SYSTEM_DATA"

    llm-wiki --config "$WIKI_RUNTIME_CONFIG" spaces create \
        "$SYSTEM_DATA" \
        --name "system" \
        --description "AI behavior and rules" || { echo "ERROR: system spaces create failed"; exit 1; }

    mkdir -p "$SYSTEM_DATA/schemas"
    for SCHEMA in main engagement messaging initiative; do
        cp "$SCHEMAS_SRC/${SCHEMA}.json" "$SYSTEM_DATA/schemas/${SCHEMA}.json"
        echo "Copied system schema: $SCHEMA"
    done

    # Seed system pages
    SEED_SRC="/wiki/seed/system"
    if [ -d "$SEED_SRC" ]; then
        mkdir -p "$SYSTEM_DATA/wiki"
        cp "$SEED_SRC"/*.md "$SYSTEM_DATA/wiki/"
        echo "Seeded system pages from $SEED_SRC"
    fi

    cd "$SYSTEM_DATA" && \
        git config user.email "$WIKI_GIT_EMAIL" && \
        git config user.name "$WIKI_GIT_NAME" && \
        git add schemas/ wiki/ && \
        git commit -m "chore: register 4 system rule schemas and seed 14 pages" && \
        cd /wiki

    echo "System wiki initialized."
else
    echo "Re-registering existing system wiki space at $SYSTEM_DATA..."
    llm-wiki --config "$WIKI_RUNTIME_CONFIG" spaces register \
        "$SYSTEM_DATA" \
        --name "system" \
        --description "AI behavior and rules" || { echo "ERROR: system spaces register failed"; exit 1; }
    echo "System wiki space re-registered."
fi

exec llm-wiki --config "$WIKI_RUNTIME_CONFIG" serve --http ":${WIKI_PORT}"
