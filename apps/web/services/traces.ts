import { API_URL } from "@/lib/constants";

export type TraceStatus = "ok" | "error" | "unset";

export interface Trace {
  id: string;
  projectId: string;
  traceId: string;
  rootService: string;
  durationMs: number;
  spanCount: number;
  status: TraceStatus;
  startedAt: string;
}

export interface Span {
  id: string;
  traceId: string;
  spanId: string;
  parentSpanId?: string;
  service: string;
  operation: string;
  durationMs: number;
  status: TraceStatus;
  attributes: Record<string, unknown>;
  startedAt: string;
}

export interface TraceDetail extends Trace {
  spans: Span[];
}

export interface TraceListResponse {
  data: Trace[];
  meta: { total: number; limit: number; offset: number };
}

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchTraces(
  projectId: string,
  params: { service?: string; status?: TraceStatus; limit?: number } = {},
  token?: string,
): Promise<TraceListResponse> {
  const query = new URLSearchParams();
  if (params.service) query.set("service", params.service);
  if (params.status) query.set("status", params.status);
  if (params.limit) query.set("limit", String(params.limit));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/traces?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) throw new Error(`Failed to fetch traces: ${response.status}`);
  return response.json();
}

export async function fetchTraceDetail(
  projectId: string,
  traceId: string,
  token?: string,
): Promise<TraceDetail> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/traces/${traceId}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) throw new Error(`Failed to fetch trace: ${response.status}`);
  return response.json();
}

export function formatDuration(ms: number): string {
  if (ms >= 1000) return `${(ms / 1000).toFixed(2)}s`;
  return `${ms.toFixed(1)}ms`;
}
