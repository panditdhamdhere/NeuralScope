"use client";

import Link from "next/link";
import { useCallback, useEffect, useState } from "react";
import {
  Activity,
  AlertTriangle,
  Bot,
  ScrollText,
  Shield,
} from "lucide-react";

import type { ProjectOverview } from "@neuralscope/shared";

import { cn } from "@/lib/utils";
import { CreateProjectCard } from "@/features/overview/create-project-card";
import { fetchProjectOverview } from "@/services/overview";

const LEVEL_STYLES: Record<string, string> = {
  error: "text-red-400",
  fatal: "text-red-500",
  warn: "text-amber-400",
  info: "text-sky-400",
  debug: "text-zinc-500",
};

interface OverviewDashboardProps {
  projectId?: string;
  token?: string;
  projectName?: string;
  signedIn?: boolean;
  sessionError?: string;
  onProjectCreated?: (project: { id: string; name: string; slug: string }) => void;
}

export function OverviewDashboard({
  projectId,
  token,
  projectName,
  signedIn,
  sessionError,
  onProjectCreated,
}: OverviewDashboardProps) {
  const [overview, setOverview] = useState<ProjectOverview>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();

  const load = useCallback(async () => {
    if (!projectId) return;

    setLoading(true);
    setError(undefined);

    try {
      const data = await fetchProjectOverview(projectId, token);
      setOverview(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load overview");
    } finally {
      setLoading(false);
    }
  }, [projectId, token]);

  useEffect(() => {
    void load();
  }, [load]);

  if (!projectId) {
    if (sessionError) {
      return (
        <div className="glass rounded-xl p-8 text-center text-sm text-red-300">
          {sessionError}
          <p className="mt-2 text-zinc-500">
            Check that the API is running at{" "}
            <code className="text-zinc-400">NEXT_PUBLIC_API_URL</code> in your{" "}
            <code className="text-zinc-400">.env</code>.
          </p>
        </div>
      );
    }

    if (signedIn && token && onProjectCreated) {
      return <CreateProjectCard token={token} onCreated={onProjectCreated} />;
    }

    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Sign in and create a project to view your observability dashboard.
      </div>
    );
  }

  if (loading && !overview) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Loading dashboard...
      </div>
    );
  }

  const stats = [
    {
      label: "Errors (24h)",
      value: overview?.errorLogs24h ?? 0,
      icon: AlertTriangle,
      color: "text-amber-400",
      href: "/logs",
    },
    {
      label: "Failed Traces",
      value: overview?.failedTraces24h ?? 0,
      icon: ScrollText,
      color: "text-red-400",
      href: "/traces",
    },
    {
      label: "Open Incidents",
      value: overview?.openIncidents ?? 0,
      icon: Shield,
      color: "text-orange-400",
      href: "/incidents",
    },
    {
      label: "AI Conversations",
      value: overview?.conversations ?? 0,
      icon: Bot,
      color: "text-violet-400",
      href: "/chat",
    },
  ];

  return (
    <div className="space-y-6">
      {projectName && (
        <p className="text-sm text-zinc-500">
          Project: <span className="text-zinc-300">{projectName}</span>
        </p>
      )}

      {error && (
        <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {stats.map((stat) => (
          <Link
            key={stat.label}
            href={stat.href}
            className="glass rounded-xl p-5 transition-colors hover:border-indigo-500/30"
          >
            <div className="flex items-center justify-between">
              <p className="text-sm text-zinc-400">{stat.label}</p>
              <stat.icon className={cn("h-4 w-4", stat.color)} />
            </div>
            <p className="mt-2 text-3xl font-bold text-white">{stat.value}</p>
          </Link>
        ))}
      </div>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <div className="glass rounded-xl p-6 lg:col-span-2">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-sm font-medium text-zinc-300">Recent Logs</h2>
            <Link href="/logs" className="text-xs text-indigo-400 hover:text-indigo-300">
              View all →
            </Link>
          </div>

          {!overview?.recentLogs.length ? (
            <p className="text-sm text-zinc-600">
              No logs yet. Ingest telemetry or visit Live Logs to get started.
            </p>
          ) : (
            <div className="space-y-2">
              {overview.recentLogs.map((log) => (
                <div
                  key={log.id}
                  className="rounded-lg border border-zinc-800/60 bg-zinc-900/40 px-3 py-2"
                >
                  <div className="flex items-center gap-2 text-xs">
                    <span
                      className={cn(
                        "uppercase",
                        LEVEL_STYLES[log.level] ?? "text-zinc-500",
                      )}
                    >
                      {log.level}
                    </span>
                    {log.service && (
                      <span className="text-zinc-600">{log.service}</span>
                    )}
                    <span className="ml-auto text-zinc-600">
                      {new Date(log.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                  <p className="mt-1 truncate text-sm text-zinc-300">{log.message}</p>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-4">
          <div className="glass rounded-xl p-6">
            <h2 className="mb-4 flex items-center gap-2 text-sm font-medium text-zinc-300">
              <Activity className="h-4 w-4 text-indigo-400" />
              System Metrics
            </h2>
            <div className="space-y-3">
              <MetricGauge
                label="CPU"
                value={overview?.cpuUsage}
                href="/metrics"
              />
              <MetricGauge
                label="Memory"
                value={overview?.memoryUsage}
                href="/metrics"
              />
            </div>
          </div>

          <div className="glass rounded-xl p-6">
            <h2 className="mb-3 text-sm font-medium text-zinc-300">Quick links</h2>
            <div className="grid grid-cols-2 gap-2 text-sm">
              {(
                [
                  ["Metrics", "/metrics"],
                  ["Network", "/network"],
                  ["Security", "/security"],
                  ["Architecture", "/architecture"],
                ] as const
              ).map(([label, href]) => (
                <Link
                  key={href}
                  href={href}
                  className="rounded-lg border border-zinc-800 px-3 py-2 text-zinc-400 hover:border-indigo-500/40 hover:text-zinc-200"
                >
                  {label}
                </Link>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function MetricGauge({
  label,
  value,
  href,
}: {
  label: string;
  value?: number;
  href: string;
}) {
  const display = value != null ? `${value.toFixed(1)}%` : "—";

  return (
    <Link href={href} className="block rounded-lg bg-zinc-900/50 p-3 hover:bg-zinc-900">
      <div className="flex items-center justify-between text-sm">
        <span className="text-zinc-400">{label}</span>
        <span className="font-medium text-white">{display}</span>
      </div>
      {value != null && (
        <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-zinc-800">
          <div
            className="h-full rounded-full bg-indigo-500 transition-all"
            style={{ width: `${Math.min(value, 100)}%` }}
          />
        </div>
      )}
    </Link>
  );
}
