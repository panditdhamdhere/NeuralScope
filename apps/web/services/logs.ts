import type { LogEntry, LogLevel } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

export interface LogSearchParams {
  level?: LogLevel;
  service?: string;
  search?: string;
  traceId?: string;
  limit?: number;
  offset?: number;
}

export interface LogListResponse {
  data: LogEntry[];
  meta: {
    total: number;
    limit: number;
    offset: number;
  };
}

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = {
    "Content-Type": "application/json",
  };

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  return headers;
}

export async function fetchLogs(
  projectId: string,
  params: LogSearchParams = {},
  token?: string,
): Promise<LogListResponse> {
  const query = new URLSearchParams();

  if (params.level) query.set("level", params.level);
  if (params.service) query.set("service", params.service);
  if (params.search) query.set("search", params.search);
  if (params.traceId) query.set("traceId", params.traceId);
  if (params.limit) query.set("limit", String(params.limit));
  if (params.offset) query.set("offset", String(params.offset));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/logs?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch logs: ${response.status}`);
  }

  return response.json();
}

export async function ingestLog(
  projectId: string,
  entry: {
    level: LogLevel;
    message: string;
    service?: string;
    traceId?: string;
    metadata?: Record<string, unknown>;
  },
  token?: string,
): Promise<LogEntry> {
  const response = await fetch(`${API_URL}/api/v1/projects/${projectId}/logs`, {
    method: "POST",
    headers: authHeaders(token),
    body: JSON.stringify(entry),
  });

  if (!response.ok) {
    throw new Error(`Failed to ingest log: ${response.status}`);
  }

  return response.json();
}

export function buildLogWebSocketUrl(projectId: string, token?: string): string {
  const base = process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:8080/ws";
  const url = new URL(base);
  url.searchParams.set("project_id", projectId);
  // Prefer session cookies when API and web share the same site (production ingress).
  // Token query param remains a fallback for local cross-origin development.
  if (token && process.env.NODE_ENV === "development") {
    url.searchParams.set("token", token);
  }
  return url.toString();
}
