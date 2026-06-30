"use client";

import { TraceExplorer } from "@/features/traces/trace-explorer";
import { useProjectSession } from "@/hooks/use-project-session";

export default function TracesPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Traces</h1>
        <p className="mt-1 text-sm text-zinc-400">
          OpenTelemetry distributed traces with span timelines.
        </p>
      </div>

      {loading ? (
        <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <TraceExplorer projectId={projectId} token={token} />
      )}
    </div>
  );
}
