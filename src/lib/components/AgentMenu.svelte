<script lang="ts">
  import { Icon, ICONS } from "$lib/icons";
  import type { AgentRecord } from "$lib/types";

  let {
    agents,
    activeId,
    onselect,
    onmanage
  }: {
    agents: AgentRecord[];
    activeId: string;
    onselect: (id: string) => void;
    onmanage: () => void;
  } = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement | null>(null);

  const active = $derived(agents.find((agent) => agent.id === activeId) ?? agents[0]);

  function closeOnOutsideClick(event: MouseEvent) {
    if (open && root && !root.contains(event.target as Node)) {
      open = false;
    }
  }

  function pick(id: string) {
    onselect(id);
    open = false;
  }

  function manage() {
    open = false;
    onmanage();
  }
</script>

<svelte:window onclick={closeOnOutsideClick} />

<div class="menu" bind:this={root}>
  <button type="button" class="trigger" aria-expanded={open} onclick={() => (open = !open)}>
    <span class="name">{active?.name ?? "Tokenmaxing"}</span>
    {#if active?.tierLabel}<span class="tier">{active.tierLabel}</span>{/if}
    <span class="caret" class:open><Icon icon={ICONS.chevron} /></span>
  </button>

  {#if open}
    <div class="list">
      {#each agents as agent (agent.id)}
        <button
          type="button"
          class="option"
          class:selected={agent.id === active?.id}
          onclick={() => pick(agent.id)}
        >
          <span class="option-name">{agent.name}</span>
          {#if agent.tierLabel}<span class="option-tier">{agent.tierLabel}</span>{/if}
          {#if agent.id === active?.id}
            <span class="check"><Icon icon={ICONS.check} /></span>
          {/if}
        </button>
      {/each}

      <button type="button" class="option manage" onclick={manage}>
        <span class="option-name">Manage sources</span>
        <span class="check"><Icon icon={ICONS.settings} /></span>
      </button>
    </div>
  {/if}
</div>

<style>
  .menu {
    position: relative;
    min-width: 0;
  }

  .trigger {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 2px 6px 2px 0;
    background: transparent;
    border: none;
    color: inherit;
  }

  .name {
    font-size: 16px;
    font-weight: 600;
    white-space: nowrap;
  }

  .tier {
    padding: 2px 7px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
  }

  .caret {
    display: grid;
    place-items: center;
    font-size: 13px;
    color: var(--text-faint);
    transition: transform 0.15s ease;
  }

  .caret.open {
    transform: rotate(180deg);
  }

  .trigger:hover .caret {
    color: var(--text);
  }

  .list {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 20;
    min-width: 220px;
    display: flex;
    flex-direction: column;
    background: var(--surface-raised);
    border: 1px solid var(--border-strong);
  }

  .option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 10px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    text-align: left;
    font-size: 12px;
    color: var(--text-muted);
  }

  .option:last-child {
    border-bottom: none;
  }

  .option:hover {
    background: var(--surface-sunken);
    color: var(--text);
  }

  .option.selected {
    color: var(--accent);
  }

  .option-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .option-tier {
    font-size: 11px;
    color: var(--text-faint);
  }

  .check {
    display: grid;
    place-items: center;
    font-size: 13px;
  }

  .manage {
    color: var(--text-faint);
  }
</style>
