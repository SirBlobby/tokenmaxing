<script lang="ts">
  import LifetimeStats from "./LifetimeStats.svelte";
  import LimitList from "./LimitList.svelte";
  import ModelBreakdown from "./ModelBreakdown.svelte";
  import StatusBanner from "./StatusBanner.svelte";
  import TodayStats from "./TodayStats.svelte";
  import TokenHistoryChart from "./TokenHistoryChart.svelte";
  import { Icon, ICONS } from "$lib/icons";
  import type { AgentRecord } from "$lib/types";

  let { agent }: { agent: AgentRecord } = $props();

  type ViewMode = "today" | "history";
  let viewMode = $state<ViewMode>("today");
</script>

<div class="panel">
  {#if agent.usageStatusText}
    <StatusBanner title={agent.usageStatusText} detail={agent.authHelpText} />
  {/if}

  <LimitList limits={agent.limits} />

  <div class="view-toggle">
    <button type="button" class:active={viewMode === "today"} onclick={() => (viewMode = "today")}>
      <Icon icon={ICONS.calendar} />
      Today
    </button>
    <button type="button" class:active={viewMode === "history"} onclick={() => (viewMode = "history")}>
      <Icon icon={ICONS.history} />
      History
    </button>
  </div>

  {#if viewMode === "today"}
    <TodayStats
      prompts={agent.stats.todayPrompts}
      sessions={agent.stats.todaySessions}
      totalTokens={agent.stats.todayTotalTokens}
    />
    <TokenHistoryChart history={agent.stats.history} compact />
    <ModelBreakdown modelUsage={agent.stats.todayModelUsage} title="Today by model" />
  {:else}
    <TokenHistoryChart history={agent.stats.history} />
    <ModelBreakdown modelUsage={agent.stats.modelUsage} />
    <LifetimeStats
      totalPrompts={agent.stats.totalPrompts}
      totalSessions={agent.stats.totalSessions}
      activeDays={agent.stats.activeDays}
    />
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .view-toggle {
    display: flex;
    border: 1px solid var(--border);
  }

  .view-toggle button {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--surface-sunken);
    border: none;
    border-right: 1px solid var(--border);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-faint);
  }

  .view-toggle button:last-child {
    border-right: none;
  }

  .view-toggle button:hover {
    color: var(--text-muted);
  }

  .view-toggle button.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
