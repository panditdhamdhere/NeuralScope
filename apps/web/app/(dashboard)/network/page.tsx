"use client";

import { NetworkGraph } from "@/features/network/network-graph";
import { useProjectSession } from "@/hooks/use-project-session";

export default function NetworkPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Network</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Interactive visualization of service-to-service connections and traffic.
        </p>
      </div>

      {loading ? (
        <div className="glass flex h-[calc(100vh-8rem)] items-center justify-center rounded-xl text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <NetworkGraph projectId={projectId} token={token} />
      )}
    </div>
  );
}
