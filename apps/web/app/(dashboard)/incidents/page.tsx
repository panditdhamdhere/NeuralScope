"use client";

import { IncidentExplorer } from "@/features/incidents/incident-explorer";
import { useProjectSession } from "@/hooks/use-project-session";

export default function IncidentsPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Incidents</h1>
        <p className="mt-1 text-sm text-zinc-400">
          AI-assisted incident reports with timelines, root cause analysis, and remediation steps.
        </p>
      </div>

      {loading ? (
        <div className="glass flex h-[calc(100vh-8rem)] items-center justify-center rounded-xl text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <IncidentExplorer projectId={projectId} token={token} />
      )}
    </div>
  );
}
