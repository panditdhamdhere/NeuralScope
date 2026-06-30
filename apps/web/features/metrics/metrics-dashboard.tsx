"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

import type { MetricPoint } from "@neuralscope/shared";

import { buildLogWebSocketUrl } from "@/services/logs";
import {
  fetchMetricNames,
  fetchMetrics,
  formatMetricValue,
} from "@/services/metrics";
import { cn } from "@/lib/utils";

const PRESET_METRICS = [
  { name: "cpu.usage", label: "CPU", color: "#818cf8", unit: "percent" as const },
  { name: "memory.usage", label: "Memory", color: "#38bdf8", unit: "percent" as const },
  { name: "disk.usage", label: "Disk", color: "#a78bfa", unit: "percent" as const },
  { name: "http.latency", label: "HTTP Latency", color: "#fbbf24", unit: "milliseconds" as const },
];

interface MetricsDashboardProps {
  projectId?: string;
  token?: string;
}

interface ChartPoint {
  time: string;
  value: number;
}

export function MetricsDashboard({ projectId, token }: MetricsDashboardProps) {
  const [selectedMetric, setSelectedMetric] = useState<string>(PRESET_METRICS[0]!.name);
  const [points, setPoints] = useState<MetricPoint[]>([]);
  const [availableNames, setAvailableNames] = useState<string[]>([]);
  const [liveValue, setLiveValue] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);

  const loadMetrics = useCallback(async () => {
    if (!projectId) return;
    setLoading(true);
    try {
      const since = new Date(Date.now() - 60 * 60 * 1000).toISOString();
      const [metricsRes, namesRes] = await Promise.all([
        fetchMetrics(projectId, { name: selectedMetric, since, limit: 500 }, token),
        fetchMetricNames(projectId, token),
      ]);
      setPoints(metricsRes.data);
      setAvailableNames(namesRes.data);
      if (metricsRes.data.length > 0) {
        setLiveValue(metricsRes.data[metricsRes.data.length - 1]!.value);
      }
    } finally {
      setLoading(false);
    }
  }, [projectId, selectedMetric, token]);

  useEffect(() => {
    loadMetrics();
  }, [loadMetrics]);

  useEffect(() => {
    if (!projectId || !token) return;

    const url = buildLogWebSocketUrl(projectId, token);
    const ws = new WebSocket(url);

    ws.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data as string);
        if (
          payload.event?.type === "metric.sample" &&
          payload.event.payload?.name === selectedMetric
        ) {
          const unit =
            PRESET_METRICS.find((m) => m.name === selectedMetric)?.unit ?? "count";
          const newPoint: MetricPoint = {
            id: crypto.randomUUID(),
            projectId,
            name: selectedMetric,
            value: payload.event.payload.value as number,
            unit,
            tags: {},
            timestamp: (payload.timestamp as string) ?? new Date().toISOString(),
          };
          setLiveValue(newPoint.value);
          setPoints((prev) => [...prev, newPoint].slice(-500));
        }
      } catch {
        // ignore malformed events
      }
    };

    return () => ws.close();
  }, [projectId, token, selectedMetric]);

  const chartData: ChartPoint[] = useMemo(
    () =>
      points.map((p) => ({
        time: new Date(p.timestamp).toLocaleTimeString(),
        value: p.value,
      })),
    [points],
  );

  const preset = PRESET_METRICS.find((m) => m.name === selectedMetric) ?? PRESET_METRICS[0]!;
  const allMetrics = [
    ...PRESET_METRICS.map((m) => m.name),
    ...availableNames.filter((n) => !PRESET_METRICS.some((p) => p.name === n)),
  ];

  if (!projectId) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Select a project to view metrics.
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        {PRESET_METRICS.map((metric) => {
          const latest = points.filter((p) => p.name === metric.name).at(-1);
          return (
            <button
              key={metric.name}
              type="button"
              onClick={() => setSelectedMetric(metric.name)}
              className={cn(
                "glass rounded-xl p-4 text-left transition-all",
                selectedMetric === metric.name && "ring-1 ring-indigo-500/50",
              )}
            >
              <p className="text-xs text-zinc-500">{metric.label}</p>
              <p className="mt-1 text-2xl font-semibold text-white">
                {latest
                  ? formatMetricValue(latest.value, latest.unit)
                  : selectedMetric === metric.name && liveValue !== null
                    ? formatMetricValue(liveValue, metric.unit)
                    : "—"}
              </p>
            </button>
          );
        })}
      </div>

      <div className="glass rounded-xl p-6">
        <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
          <h2 className="text-sm font-medium text-zinc-300">{preset.label} — Last hour</h2>
          <select
            value={selectedMetric}
            onChange={(e) => setSelectedMetric(e.target.value)}
            className="rounded-lg border border-zinc-800 bg-zinc-900/60 px-3 py-1.5 text-xs text-white"
          >
            {[...new Set(allMetrics)].map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </div>

        {loading ? (
          <p className="py-16 text-center text-sm text-zinc-500">Loading metrics...</p>
        ) : chartData.length === 0 ? (
          <p className="py-16 text-center text-sm text-zinc-500">
            No data for this metric. Ingest samples via the API to populate charts.
          </p>
        ) : (
          <ResponsiveContainer width="100%" height={320}>
            <AreaChart data={chartData}>
              <defs>
                <linearGradient id="metricGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor={preset.color} stopOpacity={0.3} />
                  <stop offset="100%" stopColor={preset.color} stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
              <XAxis dataKey="time" stroke="#71717a" tick={{ fontSize: 11 }} />
              <YAxis stroke="#71717a" tick={{ fontSize: 11 }} />
              <Tooltip
                contentStyle={{
                  background: "#18181b",
                  border: "1px solid #3f3f46",
                  borderRadius: "8px",
                  fontSize: "12px",
                }}
              />
              <Area
                type="monotone"
                dataKey="value"
                stroke={preset.color}
                fill="url(#metricGradient)"
                strokeWidth={2}
              />
            </AreaChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
