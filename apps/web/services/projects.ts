import type { Project } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

interface ListProjectsResponse {
  data: Project[];
}

export async function fetchProjects(token: string): Promise<Project[]> {
  const response = await fetch(`${API_URL}/api/v1/projects`, {
    headers: { Authorization: `Bearer ${token}` },
  });

  if (!response.ok) {
    throw new Error(`Failed to load projects (${response.status})`);
  }

  const body = (await response.json()) as ListProjectsResponse;
  return body.data ?? [];
}

export async function createProject(
  name: string,
  token: string,
): Promise<Project> {
  const response = await fetch(`${API_URL}/api/v1/projects`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ name }),
  });

  if (!response.ok) {
    const body = await response.json().catch(() => null);
    const message =
      (body as { error?: { message?: string } })?.error?.message ??
      `Failed to create project (${response.status})`;
    throw new Error(message);
  }

  return response.json() as Promise<Project>;
}
