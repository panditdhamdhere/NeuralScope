"use client";

import { useEffect, useState } from "react";
import { Pause, Play, Radio, Trash2 } from "lucide-react";

import type { LogEntry, LogLevel } from "@neuralscope/shared";

import { useLogStream } from "@/hooks/use-log-stream";
import { cn } from "@/lib/utils";
import { fetchLogs } from "@/services/logs";

const LEVEL_STYLES: Record<LogLevel, string> = {
  trace: "text-zinc-500",
  debug: "text-zinc-400",
  info: "text-sky-400",
  warn: "text-amber-400",
  error: "text-red-400",
  fatal: "text-red-500 font-semibold",
};

interface LogViewerProps {
  projectId?: string;
  token?: string;
}

export function LogViewer({ projectId, token }: LogViewerProps) {
  const [historical, setHistorical] = useState<LogEntry[]>([]);
  const [paused, setPaused] = useState(false);
  const [levelFilter, setLevelFilter] = useState<LogLevel | "all">("all");
  const [search, setSearch] = useState("");

  const { logs: liveLogs, connected, error, clear } = useLogStream({
    projectId,
    token,
    enabled: !paused && Boolean(projectId),
  });

  useEffect(() => {
    if (!projectId) return;

    fetchLogs(projectId, { limit: 50 }, token)
      .then((response) => setHistorical(response.data))
      .catch(() => setHistorical([]));
  }, [projectId, token]);

  const displayed = [
    ...liveLogs.map((log) => ({
      id: log.id,
      projectId: projectId ?? "",
      timestamp: log.timestamp,
      level: log.level,
      message: log.message,
      metadata: {},
    })),
    ...historical.filter((h) => !liveLogs.some((l) => l.id === h.id)),
  ].filter((log) => {
    if (levelFilter !== "all" && log.level !== levelFilter) return false;
    if (search && !log.message.toLowerCase().includes(search.toLowerCase())) {
      return false;
    }
    return true;
  });

  if (!projectId) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Select a project to view live logs. Sign in and create a project to get started.
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex items-center gap-2 rounded-full border border-zinc-800 bg-zinc-900/50 px-3 py-1.5 text-xs">
          <Radio
            className={cn(
              "h-3.5 w-3.5",
              connected ? "text-emerald-400 animate-pulse" : "text-zinc-600",
            )}
          />
          <span className={connected ? "text-emerald-400" : "text-zinc-500"}>
            {connected ? "Live" : "Disconnected"}
          </span>
        </div>

        <input
          type="text"
          placeholder="Search logs..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="rounded-lg border border-zinc-800 bg-zinc-900/60 px-3 py-1.5 text-sm text-white outline-none focus:border-indigo-500"
        />

        <select
          value={levelFilter}
          onChange={(e) => setLevelFilter(e.target.value as LogLevel | "all")}
          className="rounded-lg border border-zinc-800 bg-zinc-900/60 px-3 py-1.5 text-sm text-white outline-none"
        >
          <option value="all">All levels</option>
          <option value="trace">Trace</option>
          <option value="debug">Debug</option>
          <option value="info">Info</option>
          <option value="warn">Warn</option>
          <option value="error">Error</option>
          <option value="fatal">Fatal</option>
        </select>

        <button
          type="button"
          onClick={() => setPaused((p) => !p)}
          className="flex items-center gap-1.5 rounded-lg border border-zinc-800 px-3 py-1.5 text-xs text-zinc-400 hover:text-white"
        >
          {paused ? <Play className="h-3.5 w-3.5" /> : <Pause className="h-3.5 w-3.5" />}
          {paused ? "Resume" : "Pause"}
        </button>

        <button
          type="button"
          onClick={clear}
          className="flex items-center gap-1.5 rounded-lg border border-zinc-800 px-3 py-1.5 text-xs text-zinc-400 hover:text-white"
        >
          <Trash2 className="h-3.5 w-3.5" />
          Clear
        </button>
      </div>

      {error && (
        <p className="rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-300">
          {error}
        </p>
      )}

      <div className="glass overflow-hidden rounded-xl font-mono text-xs">
        <div className="max-h-[calc(100vh-16rem)] overflow-y-auto">
          {displayed.length === 0 ? (
            <p className="p-6 text-center text-zinc-500">
              No logs yet. Ingest logs via the API to see them stream here.
            </p>
          ) : (
            displayed.map((log) => (
              <div
                key={log.id}
                className="flex gap-3 border-b border-zinc-800/50 px-4 py-2 hover:bg-zinc-800/30"
              >
                <span className="shrink-0 text-zinc-600">
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <span
                  className={cn(
                    "w-12 shrink-0 uppercase",
                    LEVEL_STYLES[log.level as LogLevel] ?? "text-zinc-400",
                  )}
                >
                  {log.level}
                </span>
                <span className="text-zinc-300">{log.message}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
