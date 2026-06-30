"use client";

import { useEffect, useState } from "react";

import { LogViewer } from "@/features/logs/log-viewer";
import { useProjectSession } from "@/hooks/use-project-session";

export default function LogsPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Live Logs</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Real-time log streaming with search and level filtering.
        </p>
      </div>

      {loading ? (
        <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <LogViewer projectId={projectId} token={token} />
      )}
    </div>
  );
}
