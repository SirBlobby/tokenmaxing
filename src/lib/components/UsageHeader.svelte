<script lang="ts">
  import AgentMenu from "./AgentMenu.svelte";
  import { Icon, ICONS } from "$lib/icons";
  import { updatedLabel } from "$lib/format";
  import type { AgentRecord } from "$lib/types";

  let {
    agents,
    activeId,
    updatedAt = "",
    loading = false,
    settingsOpen = false,
    onselectagent,
    onrefresh,
    onsettings
  }: {
    agents: AgentRecord[];
    activeId: string;
    updatedAt?: string;
    loading?: boolean;
    settingsOpen?: boolean;
    onselectagent: (id: string) => void;
    onrefresh: () => void;
    onsettings: () => void;
  } = $props();

  const updated = $derived(updatedLabel(updatedAt));
</script>

<header class="header">
  {#if settingsOpen || agents.length === 0}
    <h1>{settingsOpen ? "Settings" : "Tokenmaxing"}</h1>
  {:else}
    <AgentMenu {agents} {activeId} onselect={onselectagent} onmanage={onsettings} />
  {/if}

  <div class="actions">
    {#if updated && !settingsOpen}<span class="updated">{updated}</span>{/if}
    {#if !settingsOpen}
      <button class="action" class:spinning={loading} onclick={onrefresh} aria-label="Refresh usage">
        <Icon icon={ICONS.refresh} />
      </button>
    {/if}
    <button
      class="action"
      class:active={settingsOpen}
      onclick={onsettings}
      aria-label={settingsOpen ? "Close settings" : "Open settings"}
    >
      <Icon icon={settingsOpen ? ICONS.close : ICONS.settings} />
    </button>
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .updated {
    font-size: 11px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }

  .action {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 1px solid var(--border);
    background: var(--surface-raised);
    color: var(--text-muted);
    font-size: 14px;
  }

  .action:hover {
    color: var(--text);
    border-color: var(--border-strong);
  }

  .action.active {
    color: var(--accent);
    border-color: var(--accent);
  }

  .action.spinning {
    animation: spin 0.9s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
