"use client";

import { useCallback, useEffect, useState } from "react";
import { AlertTriangle, RefreshCw, ScanSearch, ShieldAlert } from "lucide-react";

import type { SecurityFinding, Severity } from "@neuralscope/shared";

import { cn } from "@/lib/utils";
import {
  fetchSecurityFindings,
  loadSampleFindings,
  runSecurityScan,
} from "@/services/security";

const SEVERITY_STYLES: Record<Severity, string> = {
  low: "bg-zinc-800 text-zinc-400",
  medium: "bg-amber-500/15 text-amber-300",
  high: "bg-orange-500/15 text-orange-300",
  critical: "bg-red-500/15 text-red-300",
};

interface SecurityDashboardProps {
  projectId?: string;
  token?: string;
}

export function SecurityDashboard({ projectId, token }: SecurityDashboardProps) {
  const [findings, setFindings] = useState<SecurityFinding[]>([]);
  const [loading, setLoading] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string>();
  const [severityFilter, setSeverityFilter] = useState<Severity | "all">("all");
  const [scanContent, setScanContent] = useState("");

  const loadFindings = useCallback(async () => {
    if (!projectId) return;

    setLoading(true);
    setError(undefined);

    try {
      const response = await fetchSecurityFindings(
        projectId,
        severityFilter === "all" ? {} : { severity: severityFilter },
        token,
      );
      setFindings(response.data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load findings");
    } finally {
      setLoading(false);
    }
  }, [projectId, token, severityFilter]);

  useEffect(() => {
    void loadFindings();
  }, [loadFindings]);

  async function handleScan() {
    if (!projectId) return;

    setScanning(true);
    setError(undefined);

    try {
      await runSecurityScan(
        projectId,
        {
          content: scanContent.trim() || undefined,
          scanLogs: true,
        },
        token,
      );
      setScanContent("");
      await loadFindings();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Scan failed");
    } finally {
      setScanning(false);
    }
  }

  async function handleLoadSample() {
    if (!projectId) return;

    setScanning(true);
    setError(undefined);

    try {
      await loadSampleFindings(projectId, token);
      await loadFindings();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load sample");
    } finally {
      setScanning(false);
    }
  }

  if (!projectId) {
    return (
      <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
        Sign in and create a project to run security scans.
      </div>
    );
  }

  const counts = findings.reduce<Record<Severity, number>>(
    (acc, finding) => {
      acc[finding.severity] += 1;
      return acc;
    },
    { low: 0, medium: 0, high: 0, critical: 0 },
  );

  return (
    <div className="space-y-4">
      <div className="grid gap-4 md:grid-cols-4">
        {(["critical", "high", "medium", "low"] as Severity[]).map((severity) => (
          <div key={severity} className="glass rounded-xl p-4">
            <p className="text-xs capitalize text-zinc-500">{severity}</p>
            <p className="mt-1 text-2xl font-semibold text-white">{counts[severity]}</p>
          </div>
        ))}
      </div>

      <div className="glass rounded-xl p-4">
        <div className="flex flex-wrap items-center gap-3">
          <button
            type="button"
            onClick={() => void handleScan()}
            disabled={scanning}
            className="inline-flex items-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
          >
            <ScanSearch className={cn("h-4 w-4", scanning && "animate-pulse")} />
            Run scan
          </button>
          <button
            type="button"
            onClick={() => void handleLoadSample()}
            disabled={scanning}
            className="rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:bg-zinc-800 disabled:opacity-50"
          >
            Load sample findings
          </button>
          <button
            type="button"
            onClick={() => void loadFindings()}
            className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:bg-zinc-800"
          >
            <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
            Refresh
          </button>

          <select
            value={severityFilter}
            onChange={(event) =>
              setSeverityFilter(event.target.value as Severity | "all")
            }
            className="rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm text-zinc-300"
          >
            <option value="all">All severities</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>

        <textarea
          value={scanContent}
          onChange={(event) => setScanContent(event.target.value)}
          placeholder="Optional: paste config or code to scan for secrets..."
          rows={3}
          className="mt-4 w-full rounded-xl border border-zinc-800 bg-zinc-900/80 px-4 py-3 text-sm text-white placeholder:text-zinc-600 focus:border-indigo-500/50 focus:outline-none"
        />
      </div>

      {error && (
        <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      <div className="glass rounded-xl">
        {loading ? (
          <div className="p-8 text-center text-sm text-zinc-500">Loading findings...</div>
        ) : findings.length === 0 ? (
          <div className="flex flex-col items-center gap-3 p-10 text-center">
            <ShieldAlert className="h-8 w-8 text-zinc-600" />
            <p className="text-sm text-zinc-400">No security findings yet.</p>
            <p className="text-xs text-zinc-600">
              Run a scan to detect secrets, weak configs, and exposed ports.
            </p>
          </div>
        ) : (
          <div className="divide-y divide-zinc-800/60">
            {findings.map((finding) => (
              <FindingRow key={finding.id} finding={finding} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function FindingRow({ finding }: { finding: SecurityFinding }) {
  return (
    <div className="flex gap-4 p-4">
      <div className="mt-0.5">
        <AlertTriangle
          className={cn(
            "h-5 w-5",
            finding.severity === "critical" || finding.severity === "high"
              ? "text-red-400"
              : "text-amber-400",
          )}
        />
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="font-medium text-white">{finding.title}</h3>
          <span
            className={cn(
              "rounded-full px-2 py-0.5 text-xs capitalize",
              SEVERITY_STYLES[finding.severity],
            )}
          >
            {finding.severity}
          </span>
          <span className="rounded-full bg-zinc-800 px-2 py-0.5 text-xs text-zinc-500">
            {finding.findingType.replaceAll("_", " ")}
          </span>
        </div>
        <p className="mt-1 text-sm text-zinc-400">{finding.description}</p>
        <div className="mt-2 flex flex-wrap gap-3 text-xs text-zinc-600">
          {finding.resource && <span>Resource: {finding.resource}</span>}
          <span>{new Date(finding.detectedAt).toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}
