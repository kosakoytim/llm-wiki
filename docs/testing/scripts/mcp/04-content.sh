#!/usr/bin/env bash
section "4. Content (MCP)"

run_mcp      "content_read returns page body"        "Mixture of Experts" \
             wiki_content_read '{"uri":"concepts/mixture-of-experts"}'

run_mcp      "content_read includes frontmatter"     "type:" \
             wiki_content_read '{"uri":"concepts/mixture-of-experts"}'

run_mcp      "content_read with backlinks"           "backlinks" \
             wiki_content_read '{"uri":"concepts/mixture-of-experts","backlinks":true}'

run_mcp      "content_read via wiki:// URI"          "Mixture of Experts" \
             wiki_content_read '{"uri":"wiki://research/concepts/mixture-of-experts"}'

# content_write and content_new mutate the wiki — skip to preserve test state
skip "content_write" "mutates wiki state"
skip "content_new"   "mutates wiki state"
skip "content_commit" "mutates wiki state"
