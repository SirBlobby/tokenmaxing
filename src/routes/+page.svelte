<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import AgentPanel from "$lib/components/AgentPanel.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import StatusBanner from "$lib/components/StatusBanner.svelte";
  import UsageHeader from "$lib/components/UsageHeader.svelte";
  import { fetchSettings, fetchUsageReport, storeSettings } from "$lib/api";
  import { applyTheme, DEFAULT_THEME } from "$lib/theme";
  import type { Settings, SourceId, UsageReport } from "$lib/types";

  const POLL_INTERVAL_MS = 30_000;
  const RETRY_INTERVAL_MS = 5_000;
  const TRAY_REFRESH_EVENT = "usage-refresh";

  const emptySettings: Settings = {
    theme: DEFAULT_THEME,
    sources: {
      claudeCode: true,
      pi: true,
      opencode: true,
      codex: true,
      gemini: false,
      copilot: false
    }
  };

  let report = $state<UsageReport | null>(null);
  let settings = $state<Settings>(emptySettings);
  let activeAgentId = $state("");
  let showSettings = $state(false);
  let loadError = $state("");
  let loading = $state(false);

  const agents = $derived(report?.agents ?? []);
  const activeAgent = $derived(agents.find((agent) => agent.id === activeAgentId) ?? agents[0]);
  const retryAdvised = $derived(agents.some((agent) => agent.retryAdvised));

  async function load(force = false) {
    loading = true;
    try {
      report = await fetchUsageReport({ force, limitsOnly: !force });
      if (!report.agents.some((agent) => agent.id === activeAgentId)) {
        activeAgentId = report.agents[0]?.id ?? "";
      }
      loadError = "";
    } catch (failure) {
      loadError = failure instanceof Error ? failure.message : String(failure);
    } finally {
      loading = false;
    }
  }

  async function persist(next: Settings) {
    settings = next;
    applyTheme(next.theme);
    try {
      await storeSettings(next);
      await load(true);
    } catch (failure) {
      loadError = failure instanceof Error ? failure.message : String(failure);
    }
  }

  function changeTheme(theme: string) {
    persist({ ...settings, theme });
  }

  function changeSource(id: SourceId, value: boolean) {
    persist({ ...settings, sources: { ...settings.sources, [id]: value } });
  }

  onMount(() => {
    let timer: ReturnType<typeof setTimeout>;
    let stopListening: (() => void) | undefined;

    const schedule = (delay: number) => {
      timer = setTimeout(async () => {
        await load();
        schedule(retryAdvised ? RETRY_INTERVAL_MS : POLL_INTERVAL_MS);
      }, delay);
    };

    const start = async () => {
      try {
        settings = await fetchSettings();
      } catch {
        settings = emptySettings;
      }
      applyTheme(settings.theme);
      stopListening = await listen(TRAY_REFRESH_EVENT, () => load(true));
      await load();
      schedule(POLL_INTERVAL_MS);
    };

    start();
    return () => {
      clearTimeout(timer);
      stopListening?.();
    };
  });
</script>

<main>
  <UsageHeader
    {agents}
    activeId={activeAgent?.id ?? ""}
    updatedAt={report?.updatedAt ?? ""}
    {loading}
    settingsOpen={showSettings}
    onselectagent={(id) => (activeAgentId = id)}
    onrefresh={() => load(true)}
    onsettings={() => (showSettings = !showSettings)}
  />

  {#if loadError}
    <StatusBanner title="Could not read usage" detail={loadError} />
  {/if}

  {#if showSettings}
    <SettingsPanel
      {settings}
      sources={report?.sources ?? []}
      onthemechange={changeTheme}
      onsourcechange={changeSource}
    />
  {:else if activeAgent}
    <AgentPanel agent={activeAgent} />
  {:else if report}
    <p class="empty">No sources enabled. Open settings to pick one.</p>
  {:else if !loadError}
    <p class="empty">Reading local sessions...</p>
  {/if}
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    max-width: 640px;
    margin: 0 auto;
  }

  .empty {
    margin: 0;
    padding: 24px 0;
    text-align: center;
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
