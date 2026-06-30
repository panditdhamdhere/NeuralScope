"use client";

import { useCallback, useEffect, useState } from "react";

import { fetchProjects } from "@/services/projects";

interface Project {
  id: string;
  name: string;
  slug: string;
}

interface ProjectSession {
  projectId?: string;
  project?: Project;
  token?: string;
  signedIn: boolean;
  loading: boolean;
  error?: string;
  refresh: () => Promise<void>;
  setProject: (project: Project) => void;
}

export function useProjectSession(): ProjectSession {
  const [projectId, setProjectId] = useState<string>();
  const [project, setProjectState] = useState<Project>();
  const [token, setToken] = useState<string>();
  const [signedIn, setSignedIn] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(undefined);

    try {
      let sessionToken: string | undefined;

      const sessionRes = await fetch("/api/auth/get-session");
      if (sessionRes.ok) {
        const session = await sessionRes.json();
        sessionToken = session?.session?.token as string | undefined;
        setSignedIn(Boolean(sessionToken));
        setToken(sessionToken);
      } else {
        setSignedIn(false);
        setToken(undefined);
      }

      if (sessionToken) {
        const projects = await fetchProjects(sessionToken);
        const first = projects[0];
        if (first) {
          setProjectId(first.id);
          setProjectState(first);
        } else {
          setProjectId(undefined);
          setProjectState(undefined);
        }
      } else {
        setProjectId(undefined);
        setProjectState(undefined);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load session");
      setProjectId(undefined);
      setProjectState(undefined);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const setProject = useCallback((next: Project) => {
    setProjectId(next.id);
    setProjectState(next);
  }, []);

  return {
    projectId,
    project,
    token,
    signedIn,
    loading,
    error,
    refresh,
    setProject,
  };
}
