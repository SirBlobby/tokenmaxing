export interface TokenBucket {
  inputTokens: number;
  outputTokens: number;
  cacheReadInputTokens: number;
  cacheCreationInputTokens: number;
}

export interface RecentDay {
  date: string;
  messageCount: number;
}

export interface HistoryPoint {
  date: string;
  totalTokens: number;
  prompts: number;
}

export interface RateLimit {
  label: string;
  title?: string;
  percent: number;
  resetsAt: string;
}

export interface UsageStats {
  todayPrompts: number;
  todaySessions: number;
  todayTotalTokens: number;
  todayTokensByModel: Record<string, number>;
  todayModelUsage: Record<string, TokenBucket>;
  recentDays: RecentDay[];
  history: HistoryPoint[];
  modelUsage: Record<string, TokenBucket>;
  totalPrompts: number;
  totalSessions: number;
  activeDays: number;
  activeDates: string[];
}

export interface AgentRecord {
  id: string;
  name: string;
  ready: boolean;
  tierLabel: string;
  usageStatusText: string;
  authHelpText: string;
  limits: RateLimit[];
  retryAdvised: boolean;
  stats: UsageStats;
}

export interface SourceReport {
  id: string;
  label: string;
  description: string;
  supported: boolean;
  enabled: boolean;
  found: boolean;
  prompts: number;
  totalTokens: number;
}

export interface UsageReport {
  schemaVersion: number;
  updatedAt: string;
  agents: AgentRecord[];
  sources: SourceReport[];
}

export interface SourceSettings {
  claudeCode: boolean;
  pi: boolean;
  opencode: boolean;
  codex: boolean;
  gemini: boolean;
  copilot: boolean;
}

export type SourceId = keyof SourceSettings;

export interface Settings {
  theme: string;
  sources: SourceSettings;
}

export interface RefreshOptions {
  force?: boolean;
  limitsOnly?: boolean;
}
