use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use rmcp::transport::SseServer;
use rmcp::ServiceExt;

use crate::config;
use crate::engine::WikiEngine;
use crate::mcp::McpServer;

// ── serve_stdio ───────────────────────────────────────────────────────────────

async fn serve_stdio(server: McpServer) -> Result<()> {
    let transport = rmcp::transport::io::stdio();
    let service = server
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("failed to start MCP stdio server: {e}"))?;
    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("MCP stdio server error: {e}"))?;
    Ok(())
}

// ── serve_sse ─────────────────────────────────────────────────────────────────

async fn serve_sse(server: McpServer, port: u16, serve_cfg: &config::ServeConfig) -> Result<()> {
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    let max_attempts = if serve_cfg.max_restarts == 0 {
        1
    } else {
        serve_cfg.max_restarts
    };
    let mut backoff = std::time::Duration::from_secs(serve_cfg.restart_backoff as u64);
    let max_backoff = std::time::Duration::from_secs(30);

    for attempt in 1..=max_attempts {
        match SseServer::serve(addr).await {
            Ok(sse_server) => {
                tracing::info!(%addr, "SSE server listening");
                let _ct = sse_server.with_service(move || server.clone());
                tokio::signal::ctrl_c().await?;
                return Ok(());
            }
            Err(e) => {
                if attempt == max_attempts {
                    return Err(anyhow::anyhow!(
                        "SSE bind failed after {max_attempts} attempts: {e}"
                    ));
                }
                tracing::warn!(
                    %addr,
                    error = %e,
                    attempt,
                    max = max_attempts,
                    "SSE bind failed, retrying",
                );
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
        }
    }
    unreachable!()
}

// ── serve (orchestration) ─────────────────────────────────────────────────────

pub async fn serve(config_path: &std::path::Path, sse_port: Option<u16>, acp: bool) -> Result<()> {
    // 1. Build WikiEngine (loads config, mounts wikis, checks staleness)
    let manager = Arc::new(WikiEngine::build(config_path)?);

    let (wiki_count, serve_cfg, sse_enabled, resolved_port) = {
        let engine = manager.state.read().map_err(|_| anyhow::anyhow!("lock"))?;
        let count = engine.spaces.len();
        let cfg = engine.config.serve.clone();
        let sse = sse_port.is_some() || cfg.sse;
        let port = sse_port.unwrap_or(cfg.sse_port);
        (count, cfg, sse, port)
    };

    // 2. Log startup summary
    let mut transports = vec!["stdio".to_string()];
    if sse_enabled {
        transports.push(format!("sse :{resolved_port}"));
    }
    if acp {
        transports.push("acp".to_string());
    }
    tracing::info!(
        wikis = wiki_count,
        transports = %transports.join("] ["),
        "server started",
    );

    // 3. Build MCP server
    let mcp_server = McpServer::new(manager.clone());

    // 4. Heartbeat task
    if serve_cfg.heartbeat_secs > 0 {
        let interval_secs = serve_cfg.heartbeat_secs;
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(interval_secs as u64));
            loop {
                interval.tick().await;
                tracing::debug!("heartbeat");
            }
        });
    }

    // 5. Start transports
    if acp {
        let acp_manager = manager.clone();
        let max_restarts = serve_cfg.max_restarts;
        let initial_backoff_secs = serve_cfg.restart_backoff;

        let acp_thread = std::thread::spawn(move || {
            if max_restarts == 0 {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build ACP runtime");
                return rt.block_on(crate::acp::serve_acp(acp_manager));
            }

            let max_backoff = std::time::Duration::from_secs(30);
            let mut backoff = std::time::Duration::from_secs(initial_backoff_secs as u64);
            let mut restarts = 0u32;

            loop {
                let mgr = acp_manager.clone();
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build ACP runtime");
                match rt.block_on(crate::acp::serve_acp(mgr)) {
                    Ok(()) => break Ok(()),
                    Err(e) => {
                        restarts += 1;
                        tracing::error!(
                            transport = "acp",
                            error = %e,
                            restart = restarts,
                            max = max_restarts,
                            "transport crashed",
                        );
                        if restarts >= max_restarts {
                            tracing::error!("ACP max restarts reached, giving up");
                            break Err(e);
                        }
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(max_backoff);
                    }
                }
            }
        });

        if sse_enabled {
            serve_sse(mcp_server, resolved_port, &serve_cfg).await?;
        } else {
            serve_stdio(mcp_server).await?;
        }

        acp_thread
            .join()
            .map_err(|_| anyhow::anyhow!("ACP thread panicked"))??;
        Ok(())
    } else if sse_enabled {
        serve_sse(mcp_server, resolved_port, &serve_cfg).await
    } else {
        serve_stdio(mcp_server).await
    }
}
