#!/bin/sh
set -e

WIKI_DATA="/wiki/data/coordinator"
WIKI_CONFIG="/wiki/config.toml"
SCHEMAS_SRC="/wiki/schemas"

# Initialize wiki space on first boot
if [ ! -f "$WIKI_DATA/wiki.toml" ]; then
    echo "Initializing wiki space at $WIKI_DATA..."

    # Create the wiki space (initializes git repo and wiki.toml)
    llm-wiki --config "$WIKI_CONFIG" spaces create \
        "$WIKI_DATA" \
        --name coordinator \
        --description "Venus AI Coordinator — brain-modeled memory wiki" \
        --set-default

    # Copy custom schemas into the wiki's schemas dir so build_space picks them up on serve start
    mkdir -p "$WIKI_DATA/schemas"
    for SCHEMA in identity relationship preference routine project context daily_summary event episode task_context lesson; do
        cp "$SCHEMAS_SRC/${SCHEMA}.json" "$WIKI_DATA/schemas/${SCHEMA}.json"
        echo "Copied schema: $SCHEMA"
    done

    # Commit the schemas into the wiki git repo
    cd "$WIKI_DATA" && git add schemas/ && git commit -m "chore: register 11 custom brain-modeled schemas" && cd /wiki

    echo "Wiki initialized with custom schemas."
fi

exec llm-wiki --config "$WIKI_CONFIG" serve --http :8080
