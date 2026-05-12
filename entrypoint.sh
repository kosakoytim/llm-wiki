#!/bin/sh
set -e

WIKI_DATA="/wiki/data/main"
WIKI_CONFIG="/wiki/config.toml"
SCHEMAS_SRC="/wiki/schemas"

# Fix volume ownership if running as root (happens when Docker mounts override image ownership)
if [ "$(id -u)" = "0" ]; then
    chown -R wiki:wiki /wiki/data
    exec su-exec wiki "$0" "$@"
fi

# Initialize wiki space on first boot
if [ ! -f "$WIKI_DATA/wiki.toml" ]; then
    echo "Initializing wiki space at $WIKI_DATA..."

    llm-wiki --config "$WIKI_CONFIG" spaces create \
        "$WIKI_DATA" \
        --name main \
        --description "Venus AI Coordinator — brain-modeled memory wiki" \
        --set-default

    mkdir -p "$WIKI_DATA/schemas"
    for SCHEMA in identity relationship preference routine project context daily_summary event episode task_context lesson; do
        cp "$SCHEMAS_SRC/${SCHEMA}.json" "$WIKI_DATA/schemas/${SCHEMA}.json"
        echo "Copied schema: $SCHEMA"
    done

    cd "$WIKI_DATA" && git add schemas/ && git commit -m "chore: register 11 custom brain-modeled schemas" && cd /wiki

    echo "Wiki initialized with custom schemas."
fi

exec llm-wiki --config "$WIKI_CONFIG" serve --http :8080
