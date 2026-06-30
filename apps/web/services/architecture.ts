import type { GraphResponse } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchArchitectureGraph(
  projectId: string,
  token?: string,
): Promise<GraphResponse> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/architecture/graph`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch architecture graph: ${response.status}`);
  }

  return response.json();
}

export async function regenerateArchitectureGraph(
  projectId: string,
  token?: string,
): Promise<GraphResponse> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/architecture/regenerate`,
    {
      method: "POST",
      headers: authHeaders(token),
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to regenerate architecture graph: ${response.status}`);
  }

  return response.json();
}
