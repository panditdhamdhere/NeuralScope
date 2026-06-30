"use client";

import { useCallback, useEffect, useState } from "react";
import { Copy, Key, Plus, Trash2 } from "lucide-react";

import type { ApiKey } from "@neuralscope/shared";

import { useProjectSession } from "@/hooks/use-project-session";
import { cn } from "@/lib/utils";
import { fetchPlatformStatus } from "@/services/overview";
import {
  createApiKey,
  fetchApiKeys,
  revokeApiKey,
} from "@/services/settings";

interface SettingsPanelProps {
  token?: string;
}

export function SettingsPanel({ token }: SettingsPanelProps) {
  const { project, projectId } = useProjectSession();
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [newKeyName, setNewKeyName] = useState("");
  const [createdKey, setCreatedKey] = useState<string>();
  const [platformStatus, setPlatformStatus] = useState<string>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();

  const load = useCallback(async () => {
    setLoading(true);
    setError(undefined);

    try {
      const [keysRes, status] = await Promise.all([
        fetchApiKeys(token),
        fetchPlatformStatus().catch(() => null),
      ]);
      setKeys(keysRes.data);
      if (status) {
        setPlatformStatus(
          `${status.status} · ${status.environment} · ${status.uptimeSeconds}s uptime`,
        );
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load settings");
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    void load();
  }, [load]);

  async function handleCreateKey() {
    if (!newKeyName.trim()) return;

    try {
      const result = await createApiKey(newKeyName.trim(), token);
      setCreatedKey(result.key);
      setNewKeyName("");
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create key");
    }
  }

  async function handleRevoke(id: string) {
    try {
      await revokeApiKey(id, token);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to revoke key");
    }
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      {error && (
        <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      <section className="glass rounded-xl p-6">
        <h2 className="text-sm font-medium text-white">Project</h2>
        <div className="mt-4 grid gap-3 text-sm sm:grid-cols-2">
          <div className="rounded-lg bg-zinc-900/50 p-3">
            <p className="text-zinc-500">Name</p>
            <p className="mt-1 text-zinc-200">{project?.name ?? "—"}</p>
          </div>
          <div className="rounded-lg bg-zinc-900/50 p-3">
            <p className="text-zinc-500">Project ID</p>
            <p className="mt-1 truncate font-mono text-xs text-zinc-300">
              {projectId ?? "—"}
            </p>
          </div>
        </div>
      </section>

      <section className="glass rounded-xl p-6">
        <h2 className="text-sm font-medium text-white">Platform status</h2>
        <p className="mt-2 text-sm text-zinc-400">
          {platformStatus ?? (loading ? "Checking..." : "Unable to reach API")}
        </p>
      </section>

      <section className="glass rounded-xl p-6">
        <div className="mb-4 flex items-center gap-2">
          <Key className="h-4 w-4 text-indigo-400" />
          <h2 className="text-sm font-medium text-white">API Keys</h2>
        </div>

        <div className="flex gap-2">
          <input
            value={newKeyName}
            onChange={(event) => setNewKeyName(event.target.value)}
            placeholder="Key name (e.g. CI pipeline)"
            className="flex-1 rounded-lg border border-zinc-800 bg-zinc-900/80 px-3 py-2 text-sm text-white placeholder:text-zinc-600 focus:border-indigo-500/50 focus:outline-none"
          />
          <button
            type="button"
            onClick={() => void handleCreateKey()}
            className="inline-flex items-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-500"
          >
            <Plus className="h-4 w-4" />
            Create
          </button>
        </div>

        {createdKey && (
          <div className="mt-4 rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-4">
            <p className="text-xs text-emerald-300">
              Copy your new API key now — it won&apos;t be shown again.
            </p>
            <div className="mt-2 flex items-center gap-2">
              <code className="flex-1 truncate rounded bg-zinc-900 px-2 py-1 text-xs text-zinc-200">
                {createdKey}
              </code>
              <button
                type="button"
                onClick={() => void navigator.clipboard.writeText(createdKey)}
                className="rounded-lg border border-zinc-700 p-2 text-zinc-400 hover:text-white"
              >
                <Copy className="h-4 w-4" />
              </button>
            </div>
          </div>
        )}

        <div className="mt-4 divide-y divide-zinc-800/60">
          {loading ? (
            <p className="py-4 text-sm text-zinc-600">Loading keys...</p>
          ) : keys.length === 0 ? (
            <p className="py-4 text-sm text-zinc-600">No API keys yet.</p>
          ) : (
            keys.map((key) => (
              <div
                key={key.id}
                className="flex items-center justify-between gap-4 py-3"
              >
                <div>
                  <p className="text-sm font-medium text-zinc-200">{key.name}</p>
                  <p className="mt-0.5 font-mono text-xs text-zinc-600">
                    {key.keyPrefix}...
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() => void handleRevoke(key.id)}
                  className={cn(
                    "rounded-lg border border-zinc-800 p-2 text-zinc-500",
                    "hover:border-red-500/40 hover:text-red-400",
                  )}
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            ))
          )}
        </div>
      </section>

      <section className="glass rounded-xl p-6">
        <h2 className="text-sm font-medium text-white">Integrations</h2>
        <ul className="mt-3 space-y-2 text-sm text-zinc-400">
          <li>GEMINI_API_KEY — AI chat and incident reports</li>
          <li>JINA_API_KEY — Vector embeddings (optional)</li>
          <li>Configure via <code className="text-zinc-500">.env</code> for local dev or K8s secrets in production</li>
        </ul>
      </section>
    </div>
  );
}
