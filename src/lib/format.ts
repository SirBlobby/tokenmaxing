const WEEKDAY_FORMATTER = new Intl.DateTimeFormat(undefined, { weekday: "short" });
const SHORT_DATE_FORMATTER = new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric" });
const NUMERIC_DATE_FORMATTER = new Intl.DateTimeFormat("en-US", { month: "2-digit", day: "2-digit", year: "2-digit" });
const TIME_FORMATTER = new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit" });
const DAY_TIME_FORMATTER = new Intl.DateTimeFormat(undefined, {
  weekday: "short",
  hour: "numeric",
  minute: "2-digit"
});

const MINUTES_PER_DAY = 60 * 24;

export function compactTokens(value: number): string {
  const amount = Math.max(0, Math.round(value));
  if (amount < 1000) return String(amount);
  if (amount < 1_000_000) return trimZero(amount / 1000) + "K";
  if (amount < 1_000_000_000) return trimZero(amount / 1_000_000) + "M";
  return trimZero(amount / 1_000_000_000) + "B";
}

export function groupedNumber(value: number): string {
  return Math.max(0, Math.round(value)).toLocaleString();
}

export function percentLabel(fraction: number): string {
  const percent = Math.max(0, Math.min(1, fraction)) * 100;
  return (percent >= 10 ? Math.round(percent) : Math.round(percent * 10) / 10) + "%";
}

export function weekdayLabel(date: string): string {
  const parsed = parseLocalDate(date);
  return parsed ? WEEKDAY_FORMATTER.format(parsed) : "";
}

export function shortDateLabel(date: string): string {
  const parsed = parseLocalDate(date);
  return parsed ? SHORT_DATE_FORMATTER.format(parsed) : date;
}

export function numericDateLabel(date: string): string {
  const parsed = parseLocalDate(date);
  return parsed ? NUMERIC_DATE_FORMATTER.format(parsed) : date;
}

export function isToday(date: string): boolean {
  return date === localDateString(new Date());
}

export function resetLabel(resetsAt: string): string {
  if (!resetsAt) return "";
  const target = new Date(resetsAt);
  if (Number.isNaN(target.getTime())) return "";

  const minutes = Math.round((target.getTime() - Date.now()) / 60000);
  if (minutes <= 0) return "resetting";

  const clock = minutes < MINUTES_PER_DAY ? TIME_FORMATTER.format(target) : DAY_TIME_FORMATTER.format(target);
  return `resets in ${countdownLabel(minutes)} (${clock})`;
}

function countdownLabel(minutes: number): string {
  if (minutes < 60) return `${minutes}m`;
  if (minutes < MINUTES_PER_DAY) {
    const hours = Math.floor(minutes / 60);
    const remainder = minutes % 60;
    return remainder ? `${hours}h ${remainder}m` : `${hours}h`;
  }
  const days = Math.round(minutes / MINUTES_PER_DAY);
  return `${days}d`;
}

export function updatedLabel(updatedAt: string): string {
  const parsed = new Date(updatedAt);
  return Number.isNaN(parsed.getTime()) ? "" : TIME_FORMATTER.format(parsed);
}

export function modelLabel(model: string): string {
  const withoutVendor = model.replace(/^claude-/, "").replace(/-\d{8}$/, "");
  const words = withoutVendor.split(/[-_]/).filter(Boolean);
  if (words.length === 0) return model;

  const family = capitalize(words[0]);
  const version = words.slice(1).filter((word) => /^\d+$/.test(word)).join(".");
  const extras = words.slice(1).filter((word) => !/^\d+$/.test(word)).map(capitalize);
  return [family, version, ...extras].filter(Boolean).join(" ");
}

function capitalize(word: string): string {
  return word.charAt(0).toUpperCase() + word.slice(1);
}

function trimZero(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  return Number.isInteger(rounded) ? String(rounded) : rounded.toFixed(1);
}

function parseLocalDate(date: string): Date | null {
  const parts = date.split("-").map(Number);
  if (parts.length !== 3 || parts.some(Number.isNaN)) return null;
  return new Date(parts[0], parts[1] - 1, parts[2]);
}

function localDateString(value: Date): string {
  const month = String(value.getMonth() + 1).padStart(2, "0");
  const day = String(value.getDate()).padStart(2, "0");
  return `${value.getFullYear()}-${month}-${day}`;
}
