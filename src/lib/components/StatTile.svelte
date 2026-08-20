<script lang="ts">
  import { Icon, type IconName } from "$lib/icons";

  let {
    label,
    value,
    icon
  }: { label: string; value: string; icon: IconName } = $props();
</script>

<div class="tile">
  <span class="icon"><Icon {icon} /></span>
  <span class="value">{value}</span>
  <span class="label">{label}</span>
</div>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
  }

  .icon {
    display: block;
    font-size: 13px;
    color: var(--text-faint);
    line-height: 1;
    margin-bottom: 3px;
  }

  .value {
    font-size: 17px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    line-height: 1.1;
  }

  .label {
    font-size: 11px;
    color: var(--text-muted);
  }
</style>
