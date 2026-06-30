"use client";

import { useState } from "react";
import { FolderPlus } from "lucide-react";

import type { Project } from "@neuralscope/shared";

import { createProject } from "@/services/projects";

interface CreateProjectCardProps {
  token: string;
  onCreated: (project: Project) => void;
}

export function CreateProjectCard({ token, onCreated }: CreateProjectCardProps) {
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed || loading) return;

    setLoading(true);
    setError(undefined);

    try {
      const project = await createProject(trimmed, token);
      onCreated(project);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create project");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="glass mx-auto max-w-md rounded-xl p-8 text-center">
      <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-indigo-500/10">
        <FolderPlus className="h-6 w-6 text-indigo-400" />
      </div>
      <h2 className="text-lg font-medium text-white">Create your first project</h2>
      <p className="mt-2 text-sm text-zinc-400">
        Projects group logs, metrics, traces, and AI chat for one application or
        service.
      </p>

      <form onSubmit={(event) => void handleSubmit(event)} className="mt-6 space-y-4">
        <input
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="My API"
          required
          minLength={2}
          className="w-full rounded-lg border border-zinc-700 bg-zinc-900/80 px-3 py-2 text-sm text-white outline-none focus:border-indigo-500"
        />

        {error && (
          <p className="rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-300">
            {error}
          </p>
        )}

        <button
          type="submit"
          disabled={loading || name.trim().length < 2}
          className="w-full rounded-lg bg-indigo-600 py-2.5 text-sm font-medium text-white transition-colors hover:bg-indigo-500 disabled:opacity-50"
        >
          {loading ? "Creating..." : "Create project"}
        </button>
      </form>
    </div>
  );
}
