<script lang="ts">
  import Section from "./Section.svelte";
  import ToggleRow from "./ToggleRow.svelte";
  import { compactTokens, groupedNumber } from "$lib/format";
  import type { SourceId, SourceReport, SourceSettings } from "$lib/types";

  let {
    sources,
    settings,
    ontoggle
  }: {
    sources: SourceReport[];
    settings: SourceSettings;
    ontoggle: (id: SourceId, value: boolean) => void;
  } = $props();

  function statusNote(source: SourceReport): string {
    if (!source.supported) {
      return "Not supported yet";
    }
    if (!settings[source.id as SourceId]) {
      return "Disabled";
    }
    if (!source.found) {
      return "Nothing found";
    }
    return `${groupedNumber(source.prompts)} messages, ${compactTokens(source.totalTokens)} tokens`;
  }
</script>

<Section title="Sources">
  <div class="list">
    {#each sources as source (source.id)}
      <ToggleRow
        label={source.label}
        description={source.description}
        note={statusNote(source)}
        checked={settings[source.id as SourceId] ?? false}
        disabled={!source.supported}
        ontoggle={(value) => ontoggle(source.id as SourceId, value)}
      />
    {/each}
  </div>
</Section>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
</style>
