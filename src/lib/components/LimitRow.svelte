<script lang="ts">
  import type { RateLimit } from "$lib/types";
  import { percentLabel, resetLabel } from "$lib/format";

  let { limit }: { limit: RateLimit } = $props();

  const reset = $derived(resetLabel(limit.resetsAt));
  const severity = $derived(limit.percent >= 0.98 ? "danger" : limit.percent >= 0.85 ? "warn" : "normal");
</script>

<div class="limit">
  <div class="labels">
    <span class="name">{limit.title || limit.label}</span>
    <span class="value">{percentLabel(limit.percent)}</span>
  </div>

  <div class="track">
    <div class="fill {severity}" style="width: {Math.min(100, Math.max(0, limit.percent * 100))}%"></div>
  </div>

  {#if reset}<span class="reset">{reset}</span>{/if}
</div>

<style>
  .labels {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 5px;
  }

  .name {
    font-size: 12px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .value {
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--text-muted);
  }

  .track {
    height: 6px;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    transition: width 0.35s ease;
  }

  .fill.normal {
    background: var(--accent);
  }

  .fill.warn {
    background: var(--warn);
  }

  .fill.danger {
    background: var(--danger);
  }

  .reset {
    display: block;
    margin-top: 4px;
    font-size: 11px;
    color: var(--text-faint);
  }
</style>
