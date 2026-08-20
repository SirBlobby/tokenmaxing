import { invoke } from "@tauri-apps/api/core";
import type { RefreshOptions, Settings, UsageReport } from "./types";

export function fetchUsageReport(options: RefreshOptions = {}): Promise<UsageReport> {
  return invoke<UsageReport>("usage_report", {
    options: {
      force: options.force ?? false,
      limitsOnly: options.limitsOnly ?? false
    }
  });
}

export function fetchSettings(): Promise<Settings> {
  return invoke<Settings>("read_settings");
}

export function storeSettings(settings: Settings): Promise<Settings> {
  return invoke<Settings>("write_settings", { settings });
}
