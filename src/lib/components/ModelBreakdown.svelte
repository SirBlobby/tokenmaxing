<script lang="ts">
  import Section from "./Section.svelte";
  import type { TokenBucket } from "$lib/types";
  import { compactTokens, modelLabel } from "$lib/format";

  let {
    modelUsage,
    title = "All time by model"
  }: { modelUsage: Record<string, TokenBucket>; title?: string } = $props();

  const segments = [
    { key: "inputTokens", label: "Input", color: "var(--input)" },
    { key: "outputTokens", label: "Output", color: "var(--output)" },
    { key: "cacheReadInputTokens", label: "Cache read", color: "var(--cache-read)" },
    { key: "cacheCreationInputTokens", label: "Cache write", color: "var(--cache-write)" }
  ] as const;

  function bucketTotal(bucket: TokenBucket): number {
    return segments.reduce((sum, segment) => sum + (bucket[segment.key] ?? 0), 0);
  }

  const rows = $derived(
    Object.entries(modelUsage)
      .map(([model, bucket]) => ({ model, bucket, total: bucketTotal(bucket) }))
      .filter((row) => row.total > 0)
      .sort((left, right) => right.total - left.total)
  );
</script>

<Section {title}>
  {#if rows.length === 0}
    <p class="empty">No recorded usage.</p>
  {:else}
    <div class="rows">
      {#each rows as row (row.model)}
        <div class="row">
          <div class="labels">
            <span class="name">{modelLabel(row.model)}</span>
            <span class="total">{compactTokens(row.total)}</span>
          </div>
          <div class="bar">
            {#each segments as segment (segment.key)}
              {@const value = row.bucket[segment.key] ?? 0}
              {#if value > 0}
                <span
                  class="segment"
                  style="width: {(value / row.total) * 100}%; background: {segment.color}"
                  title="{segment.label}: {value.toLocaleString()}"
                ></span>
              {/if}
            {/each}
          </div>
        </div>
      {/each}
    </div>

    <ul class="legend">
      {#each segments as segment (segment.key)}
        <li>
          <span class="swatch" style="background: {segment.color}"></span>
          {segment.label}
        </li>
      {/each}
    </ul>
  {/if}
</Section>

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .labels {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 5px;
    font-size: 12px;
  }

  .name {
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .total {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .bar {
    display: flex;
    height: 6px;
    overflow: hidden;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
  }

  .segment {
    display: block;
    height: 100%;
  }

  .legend {
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    margin: 12px 0 0;
    padding: 0;
    font-size: 11px;
    color: var(--text-muted);
  }

  .legend li {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .swatch {
    width: 8px;
    height: 8px;
  }

  .empty {
    margin: 0;
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
