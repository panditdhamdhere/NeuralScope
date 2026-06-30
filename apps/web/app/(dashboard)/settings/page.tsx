"use client";

import { SettingsPanel } from "@/features/settings/settings-panel";
import { useProjectSession } from "@/hooks/use-project-session";

export default function SettingsPage() {
  const { token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-semibold text-white">Settings</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Project configuration, API keys, and integrations.
        </p>
      </div>

      {loading ? (
        <div className="glass rounded-xl p-8 text-center text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <SettingsPanel token={token} />
      )}
    </div>
  );
}
