import type { ApiKey } from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

function authHeaders(token?: string): HeadersInit {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  if (token) headers.Authorization = `Bearer ${token}`;
  return headers;
}

export async function fetchApiKeys(
  token?: string,
): Promise<{ data: ApiKey[]; meta: { total: number } }> {
  const response = await fetch(`${API_URL}/api/v1/api-keys`, {
    headers: authHeaders(token),
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch API keys: ${response.status}`);
  }

  return response.json();
}

export async function createApiKey(
  name: string,
  token?: string,
): Promise<{ apiKey: ApiKey; key: string }> {
  const response = await fetch(`${API_URL}/api/v1/api-keys`, {
    method: "POST",
    headers: authHeaders(token),
    body: JSON.stringify({ name }),
  });

  if (!response.ok) {
    throw new Error(`Failed to create API key: ${response.status}`);
  }

  return response.json();
}

export async function revokeApiKey(id: string, token?: string): Promise<void> {
  const response = await fetch(`${API_URL}/api/v1/api-keys/${id}`, {
    method: "DELETE",
    headers: authHeaders(token),
  });

  if (!response.ok) {
    throw new Error(`Failed to revoke API key: ${response.status}`);
  }
}
