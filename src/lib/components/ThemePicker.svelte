<script lang="ts">
  import Section from "./Section.svelte";
  import { Icon, ICONS } from "$lib/icons";
  import { THEME_GROUPS, themeLabel } from "$lib/theme";

  let { theme, onselect }: { theme: string; onselect: (theme: string) => void } = $props();
</script>

<Section title="Theme" hint={themeLabel(theme)}>
  <div class="groups">
    {#each THEME_GROUPS as group (group.name)}
      <div class="group">
        <span class="group-name">{group.name}</span>
        <div class="grid">
          {#each group.themes as option (option.id)}
            <button
              type="button"
              class="option"
              class:active={option.id === theme}
              onclick={() => onselect(option.id)}
            >
              <span class="swatch" style="background: {option.swatch}">
                <span class="dot" style="background: {option.accent}"></span>
              </span>
              <span class="name">{option.label}</span>
              {#if option.id === theme}
                <span class="check"><Icon icon={ICONS.check} /></span>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</Section>

<style>
  .groups {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .group-name {
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-faint);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    font-size: 12px;
  }

  .option:hover {
    border-color: var(--border-strong);
  }

  .option.active {
    border-color: var(--accent);
    color: var(--accent);
  }

  .swatch {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    flex: none;
    border: 1px solid var(--border-strong);
  }

  .dot {
    width: 8px;
    height: 8px;
  }

  .name {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .check {
    font-size: 13px;
    line-height: 1;
  }
</style>
