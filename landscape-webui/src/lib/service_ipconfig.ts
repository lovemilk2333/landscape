import type { IfaceIpModelConfig } from "@landscape-router/types/api/schemas";

export enum IfaceIpMode {
  Nothing = "nothing",
  Static = "static",
  PPPoE = "pppoe",
  DHCPClient = "dhcpclient",
}

export { type IfaceIpModelConfig };

export class IfaceIpServiceConfig {
  iface_name: string;
  enable: boolean;
  ip_model: IfaceIpModelConfig;
  update_at?: number;

  constructor(obj?: {
    iface_name?: string;
    enable?: boolean;
    ip_model?: IfaceIpModelConfig;
    update_at?: number;
  }) {
    this.iface_name = obj?.iface_name ?? "";
    this.enable = obj?.enable ?? true;
    this.update_at = obj?.update_at;
    if (obj?.ip_model !== undefined) {
      const t = obj.ip_model.t as string;
      switch (t) {
        case IfaceIpMode.Nothing:
        case IfaceIpMode.Static:
        case IfaceIpMode.PPPoE:
        case IfaceIpMode.DHCPClient:
          this.ip_model = obj.ip_model;
          break;
        default:
          this.ip_model = { t: "nothing" };
      }
    } else {
      this.ip_model = { t: "nothing" };
    }
  }
}
