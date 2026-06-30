"use client";

import { SecurityDashboard } from "@/features/security/security-dashboard";
import { useProjectSession } from "@/hooks/use-project-session";

export default function SecurityPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Security</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Secret scanning, config audit, and vulnerability detection with redacted storage.
        </p>
      </div>

      {loading ? (
        <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <SecurityDashboard projectId={projectId} token={token} />
      )}
    </div>
  );
}
