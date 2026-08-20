<script lang="ts">
  import LimitRow from "./LimitRow.svelte";
  import Section from "./Section.svelte";
  import type { RateLimit } from "$lib/types";

  let { limits, statusText = "" }: { limits: RateLimit[]; statusText?: string } = $props();
</script>

<Section title="Rate limits" hint={statusText}>
  {#if limits.length === 0}
    <p class="empty">No limits reported.</p>
  {:else}
    <div class="list">
      {#each limits as limit (limit.label)}
        <LimitRow {limit} />
      {/each}
    </div>
  {/if}
</Section>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .empty {
    margin: 0;
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
