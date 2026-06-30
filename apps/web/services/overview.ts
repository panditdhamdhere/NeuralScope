import type { ProjectOverview } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchProjectOverview(
  projectId: string,
  token?: string,
): Promise<ProjectOverview> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/overview`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch overview: ${response.status}`);
  }

  return response.json();
}

export async function fetchPlatformStatus(): Promise<{
  status: string;
  uptimeSeconds: number;
  environment: string;
}> {
  const response = await fetch(`${API_URL}/api/v1/status`);
  if (!response.ok) {
    throw new Error(`Failed to fetch status: ${response.status}`);
  }
  return response.json();
}
