import type { Incident, IncidentStatus } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchIncidents(
  projectId: string,
  params: { status?: IncidentStatus; limit?: number } = {},
  token?: string,
): Promise<{ data: Incident[]; meta: { total: number } }> {
  const query = new URLSearchParams();
  if (params.status) query.set("status", params.status);
  if (params.limit) query.set("limit", String(params.limit));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/incidents?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch incidents: ${response.status}`);
  }

  return response.json();
}

export async function fetchIncident(
  projectId: string,
  incidentId: string,
  token?: string,
): Promise<Incident> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/incidents/${incidentId}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch incident: ${response.status}`);
  }

  return response.json();
}

export async function generateIncident(
  projectId: string,
  body: { title?: string; useAi?: boolean } = {},
  token?: string,
): Promise<Incident> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/incidents/generate`,
    {
      method: "POST",
      headers: authHeaders(token),
      body: JSON.stringify(body),
    },
  );

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Failed to generate incident: ${response.status}`);
  }

  return response.json();
}

export async function updateIncidentStatus(
  projectId: string,
  incidentId: string,
  status: IncidentStatus,
  token?: string,
): Promise<Incident> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/incidents/${incidentId}`,
    {
      method: "PATCH",
      headers: authHeaders(token),
      body: JSON.stringify({ status }),
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to update incident: ${response.status}`);
  }

  return response.json();
}
