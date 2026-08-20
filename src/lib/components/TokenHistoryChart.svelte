<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    LinearScale,
    Tooltip,
    type ChartConfiguration
  } from "chart.js";
  import Section from "./Section.svelte";
  import type { HistoryPoint } from "$lib/types";
  import { compactTokens, groupedNumber, isToday, numericDateLabel, shortDateLabel, weekdayLabel } from "$lib/format";

  Chart.register(BarController, BarElement, CategoryScale, LinearScale, Tooltip);

  let { history, compact = false }: { history: HistoryPoint[]; compact?: boolean } = $props();

  type Metric = "tokens" | "messages";
  type Range = 7 | 30 | 90;

  const RANGES: { value: Range; label: string }[] = [
    { value: 7, label: "7D" },
    { value: 30, label: "30D" },
    { value: 90, label: "90D" }
  ];

  let metric = $state<Metric>("tokens");
  let range = $state<Range>(compact ? 7 : 30);
  let canvas = $state<HTMLCanvasElement | undefined>();
  let chart: Chart | undefined;

  const points = $derived(history.slice(-range));
  const total = $derived(points.reduce((sum, point) => sum + metricValue(point), 0));
  const hint = $derived(
    points.length === 0 ? "" : metric === "tokens" ? `${compactTokens(total)} tokens` : `${groupedNumber(total)} messages`
  );

  function metricValue(point: HistoryPoint): number {
    return metric === "tokens" ? point.totalTokens : point.prompts;
  }

  function cssVar(name: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  function buildConfig(): ChartConfiguration<"bar", number[], string> {
    const accent = cssVar("--accent");
    const accentSoft = cssVar("--accent-soft");
    const border = cssVar("--border");
    const textFaint = cssVar("--text-faint");
    const textMuted = cssVar("--text-muted");
    const surfaceRaised = cssVar("--surface-raised");
    const text = cssVar("--text");

    const labels = points.map((point) => (range === 7 ? weekdayLabel(point.date) : shortDateLabel(point.date)));
    const values = points.map(metricValue);
    const colors = points.map((point) => (isToday(point.date) ? accent : accentSoft));
    const borders = points.map(() => accent);

    return {
      type: "bar",
      data: {
        labels,
        datasets: [
          {
            data: values,
            backgroundColor: colors,
            borderColor: borders,
            borderWidth: 1,
            borderRadius: 0,
            categoryPercentage: range === 7 ? 0.6 : 0.9,
            barPercentage: 0.9
          }
        ]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: { duration: 200 },
        layout: { padding: 0 },
        scales: {
          x: {
            grid: { display: false },
            border: { display: false },
            ticks: {
              color: textFaint,
              font: { size: 10 },
              autoSkip: true,
              maxRotation: 0
            }
          },
          y: {
            beginAtZero: true,
            grid: { color: border },
            border: { display: false },
            ticks: {
              color: textFaint,
              font: { size: 10 },
              maxTicksLimit: 4,
              callback: (value) => (metric === "tokens" ? compactTokens(Number(value)) : String(value))
            }
          }
        },
        plugins: {
          tooltip: {
            enabled: true,
            backgroundColor: surfaceRaised,
            titleColor: text,
            bodyColor: textMuted,
            borderColor: border,
            borderWidth: 1,
            cornerRadius: 0,
            padding: 8,
            displayColors: false,
            titleFont: { size: 11, weight: "600" },
            titleMarginBottom: 6,
            bodyFont: { size: 11 },
            callbacks: {
              title: (items) => {
                const point = points[items[0]?.dataIndex ?? 0];
                return point ? numericDateLabel(point.date) : "";
              },
              label: (item) => {
                const point = points[item.dataIndex];
                const value = groupedNumber(Number(item.raw));
                if (!point) return metric === "tokens" ? `${value} tokens` : `${value} messages`;
                return metric === "tokens"
                  ? `${value} tokens · ${groupedNumber(point.prompts)} messages`
                  : `${value} messages · ${compactTokens(point.totalTokens)} tokens`;
              }
            }
          }
        }
      }
    };
  }

  function render() {
    if (!canvas) return;
    const config = buildConfig();
    if (chart) {
      chart.data = config.data;
      chart.options = config.options ?? {};
      chart.update();
    } else {
      chart = new Chart(canvas, config);
    }
  }

  $effect(() => {
    points;
    metric;
    render();
  });

  onMount(() => {
    const observer = new MutationObserver(render);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    return () => observer.disconnect();
  });

  onDestroy(() => chart?.destroy());
</script>

<Section title={compact ? "This week" : "Token history"} {hint}>
  {#if !compact}
    <div class="controls">
      <div class="toggle">
        {#each RANGES as option (option.value)}
          <button type="button" class:active={range === option.value} onclick={() => (range = option.value)}>
            {option.label}
          </button>
        {/each}
      </div>
      <div class="toggle">
        <button type="button" class:active={metric === "tokens"} onclick={() => (metric = "tokens")}>Tokens</button>
        <button type="button" class:active={metric === "messages"} onclick={() => (metric = "messages")}>
          Messages
        </button>
      </div>
    </div>
  {/if}

  {#if points.length === 0}
    <p class="empty">No recorded activity.</p>
  {:else}
    <div class="chart">
      <canvas bind:this={canvas}></canvas>
    </div>
  {/if}
</Section>

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 10px;
  }

  .toggle {
    display: flex;
    border: 1px solid var(--border);
  }

  .toggle button {
    padding: 4px 8px;
    background: var(--surface-sunken);
    border: none;
    border-right: 1px solid var(--border);
    font-size: 10px;
    letter-spacing: 0.02em;
    color: var(--text-faint);
  }

  .toggle button:last-child {
    border-right: none;
  }

  .toggle button:hover {
    color: var(--text-muted);
  }

  .toggle button.active {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .chart {
    position: relative;
    height: 140px;
  }

  .empty {
    margin: 0;
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
