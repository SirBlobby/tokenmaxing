<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    hint = "",
    children
  }: { title: string; hint?: string; children: Snippet } = $props();
</script>

<section class="section">
  <header class="section-header">
    <h2>{title}</h2>
    {#if hint}<span class="hint">{hint}</span>{/if}
  </header>
  {@render children()}
</section>

<style>
  .section {
    background: var(--surface-raised);
    border: 1px solid var(--border);
    padding: 12px 14px 14px;
  }

  .section-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 10px;
  }

  h2 {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .hint {
    font-size: 11px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }
</style>
