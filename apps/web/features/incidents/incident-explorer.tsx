"use client";

import { useCallback, useEffect, useState } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  Clock3,
  FileText,
  Sparkles,
} from "lucide-react";

import type { Incident, IncidentStatus, Severity } from "@neuralscope/shared";

import { cn } from "@/lib/utils";
import {
  fetchIncidents,
  generateIncident,
  updateIncidentStatus,
} from "@/services/incidents";

const SEVERITY_STYLES: Record<Severity, string> = {
  low: "text-zinc-400",
  medium: "text-amber-400",
  high: "text-orange-400",
  critical: "text-red-400",
};

const STATUS_STYLES: Record<IncidentStatus, string> = {
  open: "bg-red-500/15 text-red-300",
  investigating: "bg-amber-500/15 text-amber-300",
  resolved: "bg-emerald-500/15 text-emerald-300",
};

interface IncidentExplorerProps {
  projectId?: string;
  token?: string;
}

export function IncidentExplorer({ projectId, token }: IncidentExplorerProps) {
  const [incidents, setIncidents] = useState<Incident[]>([]);
  const [selected, setSelected] = useState<Incident>();
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<string>();

  const loadIncidents = useCallback(async () => {
    if (!projectId) return;

    setLoading(true);
    setError(undefined);

    try {
      const response = await fetchIncidents(projectId, {}, token);
      setIncidents(response.data);
      if (response.data[0] && !selected) {
        setSelected(response.data[0]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load incidents");
    } finally {
      setLoading(false);
    }
  }, [projectId, token]);

  useEffect(() => {
    void loadIncidents();
  }, [loadIncidents]);

  useEffect(() => {
    if (!selected && incidents.length > 0) {
      setSelected(incidents[0]);
    }
  }, [incidents, selected]);

  async function handleGenerate() {
    if (!projectId) return;

    setGenerating(true);
    setError(undefined);

    try {
      const incident = await generateIncident(projectId, { useAi: true }, token);
      setIncidents((current) => [incident, ...current]);
      setSelected(incident);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to generate report");
    } finally {
      setGenerating(false);
    }
  }

  async function handleResolve(incidentId: string) {
    if (!projectId) return;

    try {
      const updated = await updateIncidentStatus(
        projectId,
        incidentId,
        "resolved",
        token,
      );
      setIncidents((current) =>
        current.map((item) => (item.id === incidentId ? updated : item)),
      );
      setSelected(updated);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update incident");
    }
  }

  if (!projectId) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Sign in and create a project to view incident reports.
      </div>
    );
  }

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-4">
      <aside className="glass flex w-80 shrink-0 flex-col rounded-xl">
        <div className="border-b border-zinc-800/60 p-4">
          <button
            type="button"
            onClick={() => void handleGenerate()}
            disabled={generating}
            className="flex w-full items-center justify-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
          >
            <Sparkles className={cn("h-4 w-4", generating && "animate-pulse")} />
            Generate report
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-2">
          {loading ? (
            <p className="p-4 text-center text-sm text-zinc-500">Loading...</p>
          ) : incidents.length === 0 ? (
            <p className="p-4 text-center text-sm text-zinc-600">
              No incidents yet. Generate a report from your telemetry.
            </p>
          ) : (
            incidents.map((incident) => (
              <button
                key={incident.id}
                type="button"
                onClick={() => setSelected(incident)}
                className={cn(
                  "mb-1 w-full rounded-lg px-3 py-3 text-left transition-colors",
                  selected?.id === incident.id
                    ? "bg-indigo-600/15 text-indigo-200"
                    : "text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200",
                )}
              >
                <p className="truncate text-sm font-medium">{incident.title}</p>
                <div className="mt-1 flex items-center gap-2 text-xs">
                  <span className={cn("capitalize", SEVERITY_STYLES[incident.severity])}>
                    {incident.severity}
                  </span>
                  <span className="text-zinc-600">·</span>
                  <span className="capitalize text-zinc-600">{incident.status}</span>
                </div>
              </button>
            ))
          )}
        </div>
      </aside>

      <div className="glass min-w-0 flex-1 overflow-y-auto rounded-xl p-6">
        {error && (
          <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
            {error}
          </div>
        )}

        {!selected ? (
          <div className="flex h-full flex-col items-center justify-center text-center">
            <FileText className="mb-3 h-8 w-8 text-zinc-600" />
            <p className="text-sm text-zinc-500">
              Select an incident or generate a new report.
            </p>
          </div>
        ) : (
          <IncidentDetail
            incident={selected}
            onResolve={() => void handleResolve(selected.id)}
          />
        )}
      </div>
    </div>
  );
}

function IncidentDetail({
  incident,
  onResolve,
}: {
  incident: Incident;
  onResolve: () => void;
}) {
  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h2 className="text-xl font-semibold text-white">{incident.title}</h2>
          <div className="mt-2 flex flex-wrap items-center gap-2 text-sm">
            <span className={cn("capitalize", SEVERITY_STYLES[incident.severity])}>
              {incident.severity} severity
            </span>
            <span
              className={cn(
                "rounded-full px-2 py-0.5 text-xs capitalize",
                STATUS_STYLES[incident.status],
              )}
            >
              {incident.status}
            </span>
            <span className="text-zinc-600">
              {new Date(incident.createdAt).toLocaleString()}
            </span>
          </div>
        </div>

        {incident.status !== "resolved" && (
          <button
            type="button"
            onClick={onResolve}
            className="inline-flex items-center gap-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-300 hover:bg-emerald-500/20"
          >
            <CheckCircle2 className="h-4 w-4" />
            Mark resolved
          </button>
        )}
      </div>

      {incident.rootCause && (
        <section>
          <h3 className="mb-2 flex items-center gap-2 text-sm font-medium text-white">
            <AlertTriangle className="h-4 w-4 text-amber-400" />
            Root cause
          </h3>
          <p className="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4 text-sm leading-relaxed text-zinc-300">
            {incident.rootCause}
          </p>
        </section>
      )}

      {incident.affectedServices.length > 0 && (
        <section>
          <h3 className="mb-2 text-sm font-medium text-white">Affected services</h3>
          <div className="flex flex-wrap gap-2">
            {incident.affectedServices.map((service) => (
              <span
                key={service}
                className="rounded-full bg-zinc-800 px-3 py-1 text-xs text-zinc-300"
              >
                {service}
              </span>
            ))}
          </div>
        </section>
      )}

      <section>
        <h3 className="mb-3 flex items-center gap-2 text-sm font-medium text-white">
          <Clock3 className="h-4 w-4 text-indigo-400" />
          Timeline
        </h3>
        <div className="space-y-3">
          {incident.timeline.map((entry, index) => (
            <div
              key={`${entry.timestamp}-${index}`}
              className="rounded-xl border border-zinc-800 bg-zinc-900/40 p-4"
            >
              <div className="flex flex-wrap items-center gap-2 text-xs text-zinc-500">
                <span className="capitalize text-indigo-400">{entry.entryType}</span>
                <span>·</span>
                <span>{new Date(entry.timestamp).toLocaleString()}</span>
              </div>
              <p className="mt-1 text-sm font-medium text-zinc-200">{entry.title}</p>
              <p className="mt-1 text-sm text-zinc-400">{entry.detail}</p>
            </div>
          ))}
        </div>
      </section>

      {incident.suggestedFixes.length > 0 && (
        <section>
          <h3 className="mb-3 text-sm font-medium text-white">Suggested fixes</h3>
          <ul className="space-y-2">
            {incident.suggestedFixes.map((fix) => (
              <li
                key={fix}
                className="rounded-lg border border-zinc-800 bg-zinc-900/40 px-4 py-3 text-sm text-zinc-300"
              >
                {fix}
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
