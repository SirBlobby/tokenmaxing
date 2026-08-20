export const THEMES = [
  { id: "dark", label: "Modern Dark", group: "Modern", swatch: "#0f1115", accent: "#d2795a" },
  { id: "light", label: "Modern Light", group: "Modern", swatch: "#f7f7f9", accent: "#b8552f" },
  { id: "mocha", label: "Mocha", group: "Catppuccin", swatch: "#1e1e2e", accent: "#fab387" },
  { id: "macchiato", label: "Macchiato", group: "Catppuccin", swatch: "#24273a", accent: "#f5a97f" },
  { id: "frappe", label: "Frappe", group: "Catppuccin", swatch: "#303446", accent: "#ef9f76" },
  { id: "latte", label: "Latte", group: "Catppuccin", swatch: "#eff1f5", accent: "#fe640b" },
  { id: "nord", label: "Nord", group: "Popular", swatch: "#3b4252", accent: "#d08770" },
  { id: "gruvbox", label: "Gruvbox Dark", group: "Popular", swatch: "#282828", accent: "#fe8019" },
  { id: "tokyonight", label: "Tokyo Night", group: "Popular", swatch: "#1a1b26", accent: "#ff9e64" }
] as const;

export type Theme = (typeof THEMES)[number];
export type ThemeId = Theme["id"];

export const DEFAULT_THEME: ThemeId = "dark";

export const THEME_GROUPS = THEMES.reduce<{ name: string; themes: Theme[] }[]>((groups, theme) => {
  const existing = groups.find((group) => group.name === theme.group);
  if (existing) {
    existing.themes.push(theme);
  } else {
    groups.push({ name: theme.group, themes: [theme] });
  }
  return groups;
}, []);

export function isKnownTheme(theme: string): theme is ThemeId {
  return THEMES.some((entry) => entry.id === theme);
}

export function themeLabel(theme: string): string {
  return THEMES.find((entry) => entry.id === theme)?.label ?? "";
}

export function applyTheme(theme: string) {
  document.documentElement.dataset.theme = isKnownTheme(theme) ? theme : DEFAULT_THEME;
}
