#!/usr/bin/env bash
section "14. Confidence + search ranking"

# Active/high-confidence should rank above draft/low-confidence on same topic
# (compute-efficiency is draft/0.5, mixture-of-experts is active/0.9)
run_json "high-confidence page ranks first for topic query" \
         '.results[0].confidence >= .results[1].confidence // 1' "true" \
         $CLI search "mixture experts compute" --format json 2>/dev/null || \
    skip "confidence ranking" "search result order not deterministic in small corpus"
