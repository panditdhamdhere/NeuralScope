import type { MetricPoint } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

export interface MetricListResponse {
  data: MetricPoint[];
  meta: { total: number };
}

export interface MetricNamesResponse {
  data: string[];
}

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchMetrics(
  projectId: string,
  params: {
    name?: string;
    since?: string;
    until?: string;
    limit?: number;
  } = {},
  token?: string,
): Promise<MetricListResponse> {
  const query = new URLSearchParams();
  if (params.name) query.set("name", params.name);
  if (params.since) query.set("since", params.since);
  if (params.until) query.set("until", params.until);
  if (params.limit) query.set("limit", String(params.limit));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/metrics?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) throw new Error(`Failed to fetch metrics: ${response.status}`);
  return response.json();
}

export async function fetchMetricNames(
  projectId: string,
  token?: string,
): Promise<MetricNamesResponse> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/metrics/names`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) throw new Error(`Failed to fetch metric names: ${response.status}`);
  return response.json();
}

export async function ingestMetric(
  projectId: string,
  metric: {
    name: string;
    value: number;
    unit: MetricPoint["unit"];
    tags?: Record<string, string>;
  },
  token?: string,
): Promise<MetricPoint> {
  const response = await fetch(`${API_URL}/api/v1/projects/${projectId}/metrics`, {
    method: "POST",
    headers: authHeaders(token),
    body: JSON.stringify(metric),
  });

  if (!response.ok) throw new Error(`Failed to ingest metric: ${response.status}`);
  return response.json();
}

export function formatMetricValue(value: number, unit: MetricPoint["unit"]): string {
  switch (unit) {
    case "percent":
      return `${value.toFixed(1)}%`;
    case "bytes":
      if (value >= 1_073_741_824) return `${(value / 1_073_741_824).toFixed(1)} GB`;
      if (value >= 1_048_576) return `${(value / 1_048_576).toFixed(1)} MB`;
      if (value >= 1024) return `${(value / 1024).toFixed(1)} KB`;
      return `${value.toFixed(0)} B`;
    case "milliseconds":
      return `${value.toFixed(1)} ms`;
    case "requests_per_second":
      return `${value.toFixed(1)} req/s`;
    default:
      return value.toFixed(2);
  }
}
