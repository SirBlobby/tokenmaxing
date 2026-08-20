<script lang="ts">
  let {
    label,
    description = "",
    note = "",
    checked,
    disabled = false,
    ontoggle
  }: {
    label: string;
    description?: string;
    note?: string;
    checked: boolean;
    disabled?: boolean;
    ontoggle: (value: boolean) => void;
  } = $props();
</script>

<button
  type="button"
  class="row"
  class:on={checked}
  {disabled}
  aria-pressed={checked}
  onclick={() => ontoggle(!checked)}
>
  <span class="text">
    <span class="label">{label}</span>
    {#if description}<span class="description">{description}</span>{/if}
    {#if note}<span class="note">{note}</span>{/if}
  </span>
  <span class="switch"><span class="knob"></span></span>
</button>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    padding: 10px;
    text-align: left;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
  }

  .row:hover:not(:disabled) {
    border-color: var(--border-strong);
  }

  .row:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label {
    font-size: 12px;
    font-weight: 600;
  }

  .description {
    font-size: 11px;
    color: var(--text-muted);
  }

  .note {
    font-size: 11px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }

  .switch {
    flex: none;
    width: 34px;
    height: 18px;
    padding: 2px;
    background: var(--surface);
    border: 1px solid var(--border-strong);
  }

  .knob {
    display: block;
    width: 12px;
    height: 12px;
    background: var(--text-faint);
    transition: transform 0.15s ease;
  }

  .row.on .switch {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .row.on .knob {
    background: var(--accent);
    transform: translateX(16px);
  }
</style>
