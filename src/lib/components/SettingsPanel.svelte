<script lang="ts">
  import SourceToggles from "./SourceToggles.svelte";
  import ThemePicker from "./ThemePicker.svelte";
  import type { Settings, SourceId, SourceReport } from "$lib/types";

  let {
    settings,
    sources,
    onthemechange,
    onsourcechange
  }: {
    settings: Settings;
    sources: SourceReport[];
    onthemechange: (theme: string) => void;
    onsourcechange: (id: SourceId, value: boolean) => void;
  } = $props();
</script>

<div class="panel">
  <ThemePicker theme={settings.theme} onselect={onthemechange} />
  <SourceToggles {sources} settings={settings.sources} ontoggle={onsourcechange} />
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
</style>
