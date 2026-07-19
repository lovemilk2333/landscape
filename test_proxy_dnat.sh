#!/bin/bash
set -e

# 手动测试 proxy DNAT 分流
# 在目标地址启动监听: nc -lkv 127.0.0.1 1080
# 从 LAN 客户端发包: curl http://example.com

PROXY_ADDR=${1:-127.0.0.1}
PROXY_PORT=${2:-1080}
FLOW_ID=${3:-5}
# 用于模拟的源 IP（LAN 客户端的 IP）
TEST_SRC=${4:-192.168.1.100}

echo "=== Proxy DNAT 手动测试 ==="
echo "Proxy target:  $PROXY_ADDR:$PROXY_PORT"
echo "Flow ID:       $FLOW_ID"
echo "Test src IP:   $TEST_SRC"
echo ""

# 清理
nft delete table ip landscape_flow 2>/dev/null || true
nft delete table ip test_mangle 2>/dev/null || true

# 1. 创建 landscape_flow 表 (DNAT)
echo "[1/4] 创建 landscape_flow 表 + 链..."
nft add table ip landscape_flow
nft add chain ip landscape_flow prerouting { type nat hook prerouting priority -105\; }
nft add chain ip landscape_flow output { type nat hook output priority -105\; }

# DNAT 规则: 匹配 mark 低 8 位 == FLOW_ID
nft add rule ip landscape_flow prerouting \
  mark and 0x000000ff == $FLOW_ID meta l4proto '{ tcp, udp }' \
  dnat to $PROXY_ADDR:$PROXY_PORT \
  comment "landscape_flow_${FLOW_ID}"

nft add rule ip landscape_flow output \
  mark and 0x000000ff == $FLOW_ID meta l4proto '{ tcp, udp }' \
  dnat to $PROXY_ADDR:$PROXY_PORT \
  comment "landscape_flow_${FLOW_ID}"

# 2. 创建 mangle 表模拟 eBPF 打 mark (替代 eBPF flow match)
echo "[2/4] 创建 mangle mark 规则 (模拟 eBPF 打 flow mark)..."
nft add table ip test_mangle
nft add chain ip test_mangle prerouting { type filter hook prerouting priority -150\; }
nft add chain ip test_mangle output { type filter hook output priority -150\; }

# 将来自 TEST_SRC 的流量打上 flow_id mark
nft add rule ip test_mangle prerouting ip saddr $TEST_SRC meta mark set $FLOW_ID
# (可选) 路由器本机测试
nft add rule ip test_mangle output meta mark set $FLOW_ID

echo "[3/4] 验证规则:"
echo "--- landscape_flow ---"
nft list table ip landscape_flow
echo ""
echo "--- test_mangle ---"
nft list table ip test_mangle
echo ""

echo "[4/4] 测试方法:"
echo "  a) 在目标地址启动监听:     nc -lkv $PROXY_ADDR $PROXY_PORT"
echo "  b) 从 LAN 客户端发包:      curl http://example.com"
echo "  c) 观察 nc 窗口是否有连接进来"
echo "  d) tcpdump 抓包验证:       tcpdump -i any port $PROXY_PORT"
echo ""
echo "    LAN 客户端 ($TEST_SRC)"
echo "         │"
echo "    nft mangle mark → mark=5"
echo "         │"
echo "    nft prerouting mark & 0xff == 5 → DNAT $PROXY_ADDR:$PROXY_PORT"
echo "         │"
echo "    └──→ proxy 收到连接"
echo ""

echo "=== 清理: nft delete table ip landscape_flow; nft delete table ip test_mangle ==="
