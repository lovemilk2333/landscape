#ifndef __LD_NAT6_MAP_OPS_H__
#define __LD_NAT6_MAP_OPS_H__

#include "nat_common.h"
#include "nat6_static.h"

static __always_inline int get_l4_checksum_offset(u32 l4_offset, u8 l4_protocol,
                                                  u32 *l4_checksum_offset) {
    if (l4_protocol == IPPROTO_TCP) {
        *l4_checksum_offset = l4_offset + offsetof(struct tcphdr, check);
    } else if (l4_protocol == IPPROTO_UDP) {
        *l4_checksum_offset = l4_offset + offsetof(struct udphdr, check);
    } else if (l4_protocol == IPPROTO_ICMPV6) {
        *l4_checksum_offset = l4_offset + offsetof(struct icmp6hdr, icmp6_cksum);
    } else {
        return NAT_OP_ERR;
    }
    return NAT_OP_OK;
}

static __always_inline bool is_same_prefix(const u8 prefix[8], const union u_inet_addr *a,
                                           u8 npt_id_mask) {
    const u8 *b = a->bits;
    u8 prefix_mask = (u8)~npt_id_mask;
    return prefix[0] == b[0] && prefix[1] == b[1] && prefix[2] == b[2] && prefix[3] == b[3] &&
           prefix[4] == b[4] && prefix[5] == b[5] && prefix[6] == b[6] &&
           ((prefix[7] & prefix_mask) == (b[7] & prefix_mask));
}

static __always_inline struct static_nat6_mapping_value *
check_egress_static_mapping_exist(u8 ip_protocol, const struct inet_pair *pkt_ip_pair) {
    struct static_nat6_mapping_key egress_key = {0};
    struct static_nat6_mapping_value *value;
    egress_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
    egress_key.l4_protocol = ip_protocol;
    egress_key.gress = NAT_MAPPING_EGRESS;
    egress_key.prefixlen = 192;
    COPY_ADDR_FROM(egress_key.addr.all, pkt_ip_pair->src_addr.all);

    egress_key.port = pkt_ip_pair->src_port;
    value = bpf_map_lookup_elem(&nat6_static_mappings, &egress_key);
    if (value) {
        return value;
    }

    egress_key.port = 0;
    return bpf_map_lookup_elem(&nat6_static_mappings, &egress_key);
}

static __always_inline int check_ingress_mapping_exist(u8 ip_protocol,
                                                       const struct inet_pair *pkt_ip_pair,
                                                       __be64 *local_client_prefix) {
    struct static_nat6_mapping_key ingress_key = {0};
    struct static_nat6_mapping_value *value = NULL;

    __be64 dst_suffix, mapping_suffix;

    ingress_key.l3_protocol = LANDSCAPE_IPV6_TYPE;
    ingress_key.l4_protocol = ip_protocol;
    ingress_key.gress = NAT_MAPPING_INGRESS;
    ingress_key.prefixlen = 96;

    ingress_key.port = pkt_ip_pair->dst_port;
    value = bpf_map_lookup_elem(&nat6_static_mappings, &ingress_key);
    if (value) {
        goto process_mapping_value;
    }

    ingress_key.port = 0;
    value = bpf_map_lookup_elem(&nat6_static_mappings, &ingress_key);
    if (!value) {
        return NAT6_STATIC_MISS;
    }

process_mapping_value:
    if (value->addr.all[3] == 0 && value->addr.all[2] == 0) {
        return NAT6_STATIC_PASS;
    }

    if (value->addr.ip != 0) {
        COPY_ADDR_FROM(local_client_prefix, value->addr.bytes);
        return NAT6_STATIC_REPLACE;
    }

    COPY_ADDR_FROM(&mapping_suffix, value->addr.bytes + 8);
    COPY_ADDR_FROM(&dst_suffix, pkt_ip_pair->dst_addr.bits + 8);

    if (mapping_suffix == dst_suffix) {
        return NAT6_STATIC_PASS;
    }

    return NAT6_STATIC_MISS;
}

#endif /* __LD_NAT6_MAP_OPS_H__ */
