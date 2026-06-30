"use client";

import { ArchitectureDiagram } from "@/features/architecture/architecture-diagram";
import { useProjectSession } from "@/hooks/use-project-session";

export default function ArchitecturePage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Architecture</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Auto-generated service dependency graph from network traffic and traces.
        </p>
      </div>

      {loading ? (
        <div className="glass flex h-[calc(100vh-8rem)] items-center justify-center rounded-xl text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <ArchitectureDiagram projectId={projectId} token={token} />
      )}
    </div>
  );
}
