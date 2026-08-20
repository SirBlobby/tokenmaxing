<script lang="ts">
  import { Icon, ICONS } from "$lib/icons";

  let { title, detail = "" }: { title: string; detail?: string } = $props();
</script>

<div class="banner">
  <span class="icon"><Icon icon={ICONS.alert} /></span>
  <div class="text">
    <strong>{title}</strong>
    {#if detail}<p>{detail}</p>{/if}
  </div>
</div>

<style>
  .banner {
    display: flex;
    gap: 9px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-left: 3px solid var(--warn);
    background: var(--surface-raised);
  }

  .icon {
    color: var(--warn);
    font-size: 14px;
    line-height: 1.3;
  }

  .text {
    min-width: 0;
  }

  strong {
    display: block;
    font-size: 12px;
    font-weight: 600;
  }

  p {
    margin: 3px 0 0;
    font-size: 11px;
    line-height: 1.45;
    color: var(--text-muted);
  }
</style>
