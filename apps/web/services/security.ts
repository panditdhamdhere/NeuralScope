import type { ScanResult, SecurityFinding, Severity } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchSecurityFindings(
  projectId: string,
  params: { severity?: Severity; limit?: number } = {},
  token?: string,
): Promise<{ data: SecurityFinding[]; meta: { total: number } }> {
  const query = new URLSearchParams();
  if (params.severity) query.set("severity", params.severity);
  if (params.limit) query.set("limit", String(params.limit));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/security/findings?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch security findings: ${response.status}`);
  }

  return response.json();
}

export async function runSecurityScan(
  projectId: string,
  body: { content?: string; scanLogs?: boolean },
  token?: string,
): Promise<ScanResult> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/security/scan`,
    {
      method: "POST",
      headers: authHeaders(token),
      body: JSON.stringify(body),
    },
  );

  if (!response.ok) {
    throw new Error(`Security scan failed: ${response.status}`);
  }

  return response.json();
}

export async function loadSampleFindings(
  projectId: string,
  token?: string,
): Promise<ScanResult> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/security/findings/sample`,
    {
      method: "POST",
      headers: authHeaders(token),
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to load sample findings: ${response.status}`);
  }

  return response.json();
}
