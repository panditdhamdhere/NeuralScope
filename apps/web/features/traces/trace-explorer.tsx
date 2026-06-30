"use client";

import { useEffect, useState } from "react";
import { ChevronRight, Clock, Server } from "lucide-react";

import {
  fetchTraceDetail,
  fetchTraces,
  formatDuration,
  type Span,
  type Trace,
  type TraceDetail,
} from "@/services/traces";
import { cn } from "@/lib/utils";

interface TraceExplorerProps {
  projectId?: string;
  token?: string;
}

interface SpanNodeData {
  span: Span;
  depth: number;
  children: SpanNodeData[];
}

export function TraceExplorer({ projectId, token }: TraceExplorerProps) {
  const [traces, setTraces] = useState<Trace[]>([]);
  const [selected, setSelected] = useState<TraceDetail | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!projectId) return;

    fetchTraces(projectId, { limit: 50 }, token)
      .then((res) => setTraces(res.data))
      .catch(() => setTraces([]))
      .finally(() => setLoading(false));
  }, [projectId, token]);

  async function selectTrace(trace: Trace) {
    if (!projectId) return;
    const detail = await fetchTraceDetail(projectId, trace.traceId, token);
    setSelected(detail);
  }

  if (!projectId) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Select a project to view traces.
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-4 lg:grid-cols-5">
      <div className="glass overflow-hidden rounded-xl lg:col-span-2">
        <div className="border-b border-zinc-800/50 px-4 py-3">
          <h2 className="text-sm font-medium text-zinc-300">Recent Traces</h2>
        </div>
        <div className="max-h-[calc(100vh-14rem)] overflow-y-auto">
          {loading ? (
            <p className="p-6 text-center text-sm text-zinc-500">Loading traces...</p>
          ) : traces.length === 0 ? (
            <p className="p-6 text-center text-sm text-zinc-500">
              No traces yet. Ingest OpenTelemetry traces via the API.
            </p>
          ) : (
            traces.map((trace) => (
              <button
                key={trace.id}
                type="button"
                onClick={() => selectTrace(trace)}
                className={cn(
                  "flex w-full items-center gap-3 border-b border-zinc-800/30 px-4 py-3 text-left hover:bg-zinc-800/30",
                  selected?.traceId === trace.traceId && "bg-indigo-500/10",
                )}
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm text-white">{trace.rootService}</p>
                  <p className="truncate font-mono text-xs text-zinc-500">{trace.traceId}</p>
                </div>
                <div className="text-right">
                  <p className="text-xs text-zinc-400">{formatDuration(trace.durationMs)}</p>
                  <p
                    className={cn(
                      "text-xs",
                      trace.status === "error" ? "text-red-400" : "text-emerald-400",
                    )}
                  >
                    {trace.status}
                  </p>
                </div>
                <ChevronRight className="h-4 w-4 shrink-0 text-zinc-600" />
              </button>
            ))
          )}
        </div>
      </div>

      <div className="glass overflow-hidden rounded-xl lg:col-span-3">
        <div className="border-b border-zinc-800/50 px-4 py-3">
          <h2 className="text-sm font-medium text-zinc-300">Span Timeline</h2>
        </div>
        {!selected ? (
          <p className="p-8 text-center text-sm text-zinc-500">
            Select a trace to view its spans.
          </p>
        ) : (
          <div className="max-h-[calc(100vh-14rem)] space-y-3 overflow-y-auto p-4">
            <div className="flex flex-wrap gap-4 text-xs text-zinc-400">
              <span className="flex items-center gap-1">
                <Server className="h-3.5 w-3.5" />
                {selected.rootService}
              </span>
              <span className="flex items-center gap-1">
                <Clock className="h-3.5 w-3.5" />
                {formatDuration(selected.durationMs)}
              </span>
              <span>{selected.spanCount} spans</span>
            </div>
            {flattenSpanTree(buildSpanTree(selected.spans)).map(({ span, depth }) => (
              <SpanRow key={span.spanId} span={span} depth={depth} maxDuration={selected.durationMs} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function buildSpanTree(spans: Span[]): SpanNodeData[] {
  const map = new Map<string, SpanNodeData>();

  for (const span of spans) {
    map.set(span.spanId, { span, depth: 0, children: [] });
  }

  const roots: SpanNodeData[] = [];

  for (const span of spans) {
    const node = map.get(span.spanId)!;
    if (span.parentSpanId && map.has(span.parentSpanId)) {
      const parent = map.get(span.parentSpanId)!;
      node.depth = parent.depth + 1;
      parent.children.push(node);
    } else {
      roots.push(node);
    }
  }

  return roots;
}

function flattenSpanTree(nodes: SpanNodeData[]): SpanNodeData[] {
  const result: SpanNodeData[] = [];

  function walk(node: SpanNodeData) {
    result.push(node);
    for (const child of node.children) {
      walk(child);
    }
  }

  for (const node of nodes) {
    walk(node);
  }

  return result;
}

function SpanRow({
  span,
  depth,
  maxDuration,
}: {
  span: Span;
  depth: number;
  maxDuration: number;
}) {
  const widthPercent = Math.max((span.durationMs / maxDuration) * 100, 2);

  return (
    <div className="space-y-1" style={{ paddingLeft: depth * 16 }}>
      <div className="flex items-center justify-between gap-2 text-xs">
        <span className="truncate text-zinc-300">
          {span.service} — {span.operation}
        </span>
        <span className="shrink-0 text-zinc-500">{formatDuration(span.durationMs)}</span>
      </div>
      <div className="h-2 rounded-full bg-zinc-800">
        <div
          className={cn(
            "h-2 rounded-full",
            span.status === "error" ? "bg-red-500/70" : "bg-indigo-500/70",
          )}
          style={{ width: `${widthPercent}%` }}
        />
      </div>
    </div>
  );
}
