import IconifyIcon, { addIcon } from "@iconify/svelte";
import { getIconData } from "@iconify/utils";
import { icons as lucide } from "@iconify-json/lucide";

export const ICONS = {
  refresh: "refresh-cw",
  message: "message-square",
  sessions: "layers",
  tokens: "coins",
  calendar: "calendar-days",
  history: "clock",
  alert: "triangle-alert",
  settings: "settings",
  close: "x",
  palette: "palette",
  plug: "plug",
  check: "check",
  chevron: "chevron-down"
} as const;

export type IconName = (typeof ICONS)[keyof typeof ICONS];

for (const name of Object.values(ICONS)) {
  const data = getIconData(lucide, name);
  if (data) {
    addIcon(name, data);
  }
}

export const Icon = IconifyIcon;
