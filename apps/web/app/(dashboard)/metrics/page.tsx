"use client";

import { MetricsDashboard } from "@/features/metrics/metrics-dashboard";
import { useProjectSession } from "@/hooks/use-project-session";

export default function MetricsPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Metrics</h1>
        <p className="mt-1 text-sm text-zinc-400">
          CPU, memory, disk, HTTP latency, and custom application metrics.
        </p>
      </div>

      {loading ? (
        <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <MetricsDashboard projectId={projectId} token={token} />
      )}
    </div>
  );
}
