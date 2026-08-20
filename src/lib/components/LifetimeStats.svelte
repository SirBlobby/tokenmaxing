<script lang="ts">
  import Section from "./Section.svelte";
  import StatTile from "./StatTile.svelte";
  import { ICONS } from "$lib/icons";
  import { groupedNumber } from "$lib/format";

  let {
    totalPrompts,
    totalSessions,
    activeDays
  }: { totalPrompts: number; totalSessions: number; activeDays: number } = $props();
</script>

<Section title="All time">
  <div class="tiles">
    <StatTile label="Messages" value={groupedNumber(totalPrompts)} icon={ICONS.message} />
    <StatTile label="Sessions" value={groupedNumber(totalSessions)} icon={ICONS.sessions} />
    <StatTile label="Active days" value={groupedNumber(activeDays)} icon={ICONS.calendar} />
  </div>
</Section>

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
</style>
