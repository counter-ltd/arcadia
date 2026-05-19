pub const NAME: &str = "lan";

mod config;
mod discovery;
mod handlers;
mod peers;
mod protocol;

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use serde::Serialize;

use crate::modules::ModuleCommand;

use config::{is_identifier_approved, load_node_config, resolve_alias_target};
use discovery::resolve_target;
use peers::node_state;
use protocol::PeerStatus;
use protocol::{
    DEFAULT_REMOTE_TIMEOUT_MS, DISCOVERY_PORT, NODE_EXEC_PREFIX, NODE_EXEC_RESULT_PREFIX,
    RECV_BUF_LARGE,
};

pub use discovery::{discover_lan_peers, set_hostname_override, start_service, stop_service};

pub struct LanServiceInfo {
    pub running: bool,
    pub port: u16,
    pub hostname: String,
    pub module_enabled: bool,
}

pub fn lan_service_info() -> LanServiceInfo {
    use std::sync::atomic::Ordering;
    LanServiceInfo {
        running: discovery::SERVICE_RUNNING.load(Ordering::SeqCst),
        port: protocol::DISCOVERY_PORT,
        hostname: discovery::local_hostname(),
        module_enabled: discovery::lan_enabled(),
    }
}

/// Row for LAN Nodes UI (desktop / callers); mirrors in-memory peer records.
#[derive(Clone, Debug)]
pub struct LanKnownPeer {
    pub ip: String,
    pub hostname: String,
    pub status: &'static str,
}

pub fn list_known_lan_peers() -> Vec<LanKnownPeer> {
    let Ok(guard) = peers::node_state().lock() else {
        return Vec::new();
    };
    guard
        .peers
        .values()
        .map(|p| LanKnownPeer {
            ip: p.ip.clone(),
            hostname: p.hostname.clone(),
            status: p.status.as_str(),
        })
        .collect()
}

/// Connected peers approved in local node config (`lan_nodes.toml`), sorted by IP.
pub fn connected_approved_session_peers() -> Vec<(String, String)> {
    let Ok(cfg) = load_node_config() else {
        return Vec::new();
    };
    let Ok(guard) = peers::node_state().lock() else {
        return Vec::new();
    };
    let mut out: Vec<(String, String)> = guard
        .peers
        .values()
        .filter(|p| {
            matches!(p.status, PeerStatus::Connected)
                && is_identifier_approved(&cfg, &p.ip, &p.hostname)
        })
        .map(|p| (p.ip.clone(), p.hostname.clone()))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[derive(Serialize)]
struct SessionTargetRow {
    ip: String,
    hostname: String,
}

fn session_targets(_args: &[&str], _ctx: &crate::modules::ExecutionContext) -> String {
    let rows: Vec<SessionTargetRow> = connected_approved_session_peers()
        .into_iter()
        .map(|(ip, hostname)| SessionTargetRow { ip, hostname })
        .collect();
    serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string())
}

pub fn execute_remote_command(
    target: &str,
    token: &str,
    args: &[&str],
    timeout_ms: Option<u64>,
) -> Result<String, String> {
    let cfg = load_node_config().map_err(|_| "Failed to load LAN node config".to_string())?;
    let resolved_target = resolve_alias_target(target, &cfg);
    let addr = resolve_target(&resolved_target)?;
    let ip = addr.ip().to_string();

    let guard = node_state()
        .lock()
        .map_err(|_| "Failed to access node state".to_string())?;
    let Some(peer) = guard.peers.get(&ip) else {
        return Err(format!("Target {target} is not a known node"));
    };
    if !matches!(peer.status, PeerStatus::Connected) {
        return Err(format!("Target {target} is not connected"));
    }
    if !is_identifier_approved(&cfg, &peer.ip, &peer.hostname) {
        return Err(format!("Target {target} is not approved"));
    }
    drop(guard);

    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        .map_err(|err| format!("Failed to create UDP socket: {err}"))?;
    let _ = socket.set_read_timeout(Some(Duration::from_millis(
        timeout_ms.unwrap_or(DEFAULT_REMOTE_TIMEOUT_MS),
    )));

    let mut payload = format!("{NODE_EXEC_PREFIX}\t{token}");
    for arg in args {
        payload.push('\t');
        payload.push_str(arg);
    }

    socket
        .send_to(
            payload.as_bytes(),
            SocketAddrV4::new(*addr.ip(), DISCOVERY_PORT),
        )
        .map_err(|err| format!("Failed to send remote command: {err}"))?;

    let mut buf = [0_u8; RECV_BUF_LARGE];
    let (len, _) = socket
        .recv_from(&mut buf)
        .map_err(|_| "Remote command timed out".to_string())?;
    let payload = String::from_utf8_lossy(&buf[..len]);
    let Some((prefix, result)) = payload.split_once('\t') else {
        return Err("Invalid remote response".to_string());
    };
    if prefix != NODE_EXEC_RESULT_PREFIX {
        return Err("Unexpected remote response".to_string());
    }
    Ok(result.to_string())
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "scan",
            description: "discover Arcadia LAN peers (--range, --self supported)",
            required_permissions: &["network.lan"],
            run: discovery::scan,
        },
        ModuleCommand {
            name: "status",
            description: "show LAN service status, port, and local hostname",
            required_permissions: &["network.lan"],
            run: discovery::service_status,
        },
        ModuleCommand {
            name: "node",
            description: "manage LAN nodes: pair|connect|accept|reject|alias|save|auto|status",
            required_permissions: &["network.lan"],
            run: handlers::node,
        },
        ModuleCommand {
            name: "session_targets",
            description: "JSON [{\"ip\",\"hostname\"}] for connected approved LAN peers (remote route picker)",
            required_permissions: &["network.lan"],
            run: session_targets,
        },
    ]
}

#[derive(Default)]
pub struct LanExtension;

impl crate::extension::Extension for LanExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "network".to_string(),
            version: "1.0.0".to_string(),
            description: "Local network discovery and peer communication.".to_string(),
            accent: String::new(),
            required_modules: vec![crate::config::modules::NET_MODULE_NAME.to_string()],
            required_permissions: vec!["network.lan".to_string()],
            workspace_permissions: Vec::new(),
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> Vec<crate::extension::OwnedModuleCommand> {
        commands()
            .iter()
            .map(crate::extension::OwnedModuleCommand::from_static)
            .collect()
    }

    fn services(&self) -> &'static [crate::services::ServiceDefinition] {
        LAN_SERVICES
    }

    fn nav_pages(&self) -> &'static [crate::navigation::NavigationPageDefinition] {
        use crate::navigation::{NavigationPageDefinition, PageLayoutKind};
        static PAGES: &[NavigationPageDefinition] = &[NavigationPageDefinition {
            id: "network.nodes",
            title: "Nodes",
            description: "Discover LAN peers and manage pairing with lan.scan / lan.node.",
            glyph: "nodes",
            system_image: "wifi",
            accent: "cyan",
            required_module: Some(NAME),
            layout_kind: PageLayoutKind::Standard,
            blocks_platform_goto: false,
        }];
        PAGES
    }

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::PermissionDefinition;
        static PERMS: &[PermissionDefinition] = &[PermissionDefinition {
            id: "network.lan",
            title: "LAN access",
            description: "Discovery, peer I/O, and LAN module commands (multicast, pairing, etc.).",
            default_global: false,
            system_grant: None,
        }];
        PERMS
    }
}

crate::register_extension!(LanExtension);

// ─── Service registration ───────────────────────────────────────────────────

fn svc_status() -> crate::services::ServiceRuntimeStatus {
    let info = lan_service_info();
    let detail = if info.running {
        format!("UDP :{} · {}", info.port, info.hostname)
    } else {
        format!("UDP :{} · stopped", info.port)
    };
    crate::services::ServiceRuntimeStatus {
        running: Some(info.running),
        detail,
    }
}

fn svc_start() -> Result<(), String> {
    start_service()
}

fn svc_stop() {
    stop_service();
}

/// LAN discovery service, advertised on the Services page.
static LAN_SERVICES: &[crate::services::ServiceDefinition] =
    &[crate::services::ServiceDefinition {
        id: "lan.discovery",
        page_id: "utility.services",
        title: "LAN Discovery",
        description: "Auto-advertises this node and discovers Arcadia peers on UDP broadcast.",
        required_module: NAME,
        glyph: "nodes",
        system_image: "wifi",
        accent: "cyan",
        controls: crate::services::ServiceControls {
            status_detail: Some(svc_status),
            start: Some(svc_start),
            stop: Some(svc_stop),
            port_for_collision: Some(|| lan_service_info().port),
        },
    }];
