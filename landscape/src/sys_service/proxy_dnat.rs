use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::Command;
use std::sync::Mutex;

/// Tracks which flow_ids currently have proxy targets, so we can clean up
/// stale entries when flow rules are removed or disabled.
static ACTIVE_PROXY_FLOWS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// Manages nftables DNAT rules for flow proxy targets.
///
/// Uses a dedicated table `landscape_flow` with `prerouting` and `output`
/// chains (type nat). Each rule matches on the flow_id bits in `skb->mark`
/// and DNATs to the proxy address:port.
///
/// Two chains are needed because:
///   - prerouting  → handles **forwarded** traffic (LAN clients)
///   - output      → handles **locally-generated** traffic (router itself)
///
/// Table layout:
///   table ip landscape_flow {
///     chain prerouting {
///       type nat hook prerouting priority -105;
///       mark & 0xff == 5 dnat to 192.168.1.100:1080 comment "landscape_flow_5"
///     }
///     chain output {
///       type nat hook output priority -105;
///       mark & 0xff == 5 dnat to 192.168.1.100:1080 comment "landscape_flow_5"
///     }
///   }
const NFT_TABLE: &str = "landscape_flow";
const NFT_PREROUTING: &str = "prerouting";
const NFT_OUTPUT: &str = "output";
const NFT_PRIORITY: &str = "-105";

const ALL_CHAINS: [&str; 2] = [NFT_PREROUTING, NFT_OUTPUT];

/// Reconcile proxy DNAT rules: enable rules for `active`, remove rules for any
/// flow that was previously active but is no longer in `active`.
///
/// Returns the list of flow_ids that were active before but are no longer,
/// so the caller can also clean up the BPF proxy map entries.
pub fn sync_proxy_flows(active: &[u32]) -> Vec<u32> {
    let mut prev = ACTIVE_PROXY_FLOWS.lock().unwrap();
    let stale: Vec<u32> = prev.iter().filter(|f| !active.contains(f)).copied().collect();
    for flow_id in &stale {
        del_proxy_dnat(*flow_id);
    }
    *prev = active.to_vec();
    stale
}

pub fn init_table() {
    let table = format!("ip {}", NFT_TABLE);
    let _ = Command::new("nft").args(["add", "table", &table]).output();
    for chain in &ALL_CHAINS {
        let _ = Command::new("nft")
            .args([
                "add", "chain", &table, chain, "{", "type", "nat", "hook", chain, "priority",
                NFT_PRIORITY, ";", "}",
            ])
            .output();
    }
}

pub fn cleanup_table() {
    let table = format!("ip {}", NFT_TABLE);
    let _ = Command::new("nft").args(["delete", "table", &table]).output();
}

fn rule_comment(flow_id: u32) -> String {
    format!("landscape_flow_{}", flow_id)
}

fn add_rule_to_all_chains(
    table: &str,
    flow_id: u32,
    dnat_target: &str,
    comment: &str,
) {
    for chain in &ALL_CHAINS {
        let status = Command::new("nft")
            .args([
                "add", "rule", table, chain, "mark", "and", "0x000000ff", "==",
                &flow_id.to_string(), "dnat", "to", dnat_target, "comment", comment,
            ])
            .output();
        match status {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("nft add dnat rule failed for chain {chain} flow {flow_id}: {stderr}");
                } else {
                    tracing::info!("nft dnat rule added to {chain} for flow {flow_id} -> {dnat_target}");
                }
            }
            Err(e) => tracing::error!("nft command failed for chain {chain}: {e}"),
        }
    }
}

pub fn set_proxy_dnat_v4(flow_id: u32, addr: Ipv4Addr, port: u16) {
    init_table();
    let table = format!("ip {}", NFT_TABLE);
    let comment = rule_comment(flow_id);
    // Remove existing rules first
    del_proxy_dnat(flow_id);
    add_rule_to_all_chains(&table, flow_id, &format!("{}:{}", addr, port), &comment);
}

pub fn set_proxy_dnat_v6(flow_id: u32, addr: Ipv6Addr, port: u16) {
    init_table();
    let table = format!("ip {}", NFT_TABLE);
    let comment = rule_comment(flow_id);
    del_proxy_dnat(flow_id);
    add_rule_to_all_chains(&table, flow_id, &format!("[{}]:{}", addr, port), &comment);
}

pub fn del_proxy_dnat(flow_id: u32) {
    let table = format!("ip {}", NFT_TABLE);
    let comment = rule_comment(flow_id);
    for chain in &ALL_CHAINS {
        let output = match Command::new("nft")
            .args(["--json", "list", "chain", &table, chain])
            .output()
        {
            Ok(o) => o,
            Err(_) => continue,
        };
        if !output.status.success() {
            continue;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let handles = extract_rule_handles_by_comment(&stdout, &comment);
        for handle in handles {
            let _ = Command::new("nft")
                .args(["delete", "rule", &table, chain, "handle", &handle.to_string()])
                .output();
        }
    }
}

/// Parse nftables JSON output to find rule handles matching the given comment.
fn extract_rule_handles_by_comment(json: &str, target_comment: &str) -> Vec<u64> {
    let mut handles = Vec::new();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
    let Ok(parsed) = parsed else {
        return handles;
    };
    let rules = parsed.get("nftables").and_then(|v| v.as_array());
    let Some(rules) = rules else {
        return handles;
    };
    for rule in rules {
        let rule_obj = match rule.as_object() {
            Some(o) => o,
            None => continue,
        };
        let rule_data = match rule_obj.get("rule") {
            Some(v) => v.as_object(),
            None => continue,
        };
        let Some(rule_data) = rule_data else {
            continue;
        };
        let handle = match rule_data.get("handle") {
            Some(v) => v.as_u64(),
            None => continue,
        };
        let Some(handle) = handle else {
            continue;
        };
        let user_comment = rule_data.get("comment").and_then(|v| v.as_str()).unwrap_or("");
        if user_comment == target_comment {
            handles.push(handle);
        }
    }
    handles
}
