<script lang="ts">
  import Section from "./Section.svelte";
  import StatTile from "./StatTile.svelte";
  import { ICONS } from "$lib/icons";
  import { compactTokens, groupedNumber } from "$lib/format";

  let {
    prompts,
    sessions,
    totalTokens
  }: {
    prompts: number;
    sessions: number;
    totalTokens: number;
  } = $props();
</script>

<Section title="Today">
  <div class="tiles">
    <StatTile label="Messages" value={groupedNumber(prompts)} icon={ICONS.message} />
    <StatTile label="Sessions" value={groupedNumber(sessions)} icon={ICONS.sessions} />
    <StatTile label="Tokens" value={compactTokens(totalTokens)} icon={ICONS.tokens} />
  </div>
</Section>

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
</style>
