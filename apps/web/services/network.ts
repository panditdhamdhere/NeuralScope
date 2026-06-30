import type { GraphResponse, NetworkEvent } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchNetworkGraph(
  projectId: string,
  token?: string,
): Promise<GraphResponse> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/network/graph`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch network graph: ${response.status}`);
  }

  return response.json();
}

export async function fetchNetworkEvents(
  projectId: string,
  params: { source?: string; destination?: string; limit?: number } = {},
  token?: string,
): Promise<{ data: NetworkEvent[]; meta: { total: number } }> {
  const query = new URLSearchParams();
  if (params.source) query.set("source", params.source);
  if (params.destination) query.set("destination", params.destination);
  if (params.limit) query.set("limit", String(params.limit));

  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/network/events?${query.toString()}`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch network events: ${response.status}`);
  }

  return response.json();
}

export async function ingestSampleNetworkEvents(
  projectId: string,
  token?: string,
): Promise<void> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/network/events/batch`,
    {
      method: "POST",
      headers: authHeaders(token),
      body: JSON.stringify([
        {
          sourceName: "browser",
          sourceType: "browser",
          destinationName: "api-gateway",
          destinationType: "service",
          protocol: "https",
          bytesSent: 2048,
          bytesReceived: 8192,
          latencyMs: 45,
        },
        {
          sourceName: "api-gateway",
          sourceType: "service",
          destinationName: "auth-service",
          destinationType: "service",
          protocol: "http",
          bytesSent: 1024,
          bytesReceived: 512,
          latencyMs: 12,
        },
        {
          sourceName: "api-gateway",
          sourceType: "service",
          destinationName: "users-service",
          destinationType: "service",
          protocol: "http",
          bytesSent: 4096,
          bytesReceived: 16384,
          latencyMs: 28,
        },
        {
          sourceName: "users-service",
          sourceType: "service",
          destinationName: "postgres",
          destinationType: "database",
          protocol: "tcp",
          bytesSent: 8192,
          bytesReceived: 32768,
          latencyMs: 3,
        },
        {
          sourceName: "users-service",
          sourceType: "service",
          destinationName: "redis",
          destinationType: "cache",
          protocol: "tcp",
          bytesSent: 512,
          bytesReceived: 256,
          latencyMs: 1,
        },
      ]),
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to ingest sample data: ${response.status}`);
  }
}
