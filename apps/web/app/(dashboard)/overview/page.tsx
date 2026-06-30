"use client";

import { OverviewDashboard } from "@/features/overview/overview-dashboard";
import { useProjectSession } from "@/hooks/use-project-session";

export default function OverviewPage() {
  const {
    projectId,
    project,
    token,
    signedIn,
    loading,
    error,
    setProject,
  } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold text-white">Overview</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Real-time snapshot of your application health and observability signals.
        </p>
      </div>

      {loading ? (
        <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <OverviewDashboard
          projectId={projectId}
          token={token}
          projectName={project?.name}
          signedIn={signedIn}
          sessionError={error}
          onProjectCreated={setProject}
        />
      )}
    </div>
  );
}
