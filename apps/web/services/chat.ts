import type {
  ChatCompletionRequest,
  ChatCompletionResponse,
  ChatConversation,
  ChatMessageRecord,
} from "@neuralscope/shared";

import { API_URL } from "@/lib/constants";

export interface ChatListResponse<T> {
  data: T[];
  meta: { total: number };
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

async function parseError(response: Response): Promise<string> {
  try {
    const body = await response.json();
    return body?.error?.message ?? `Request failed: ${response.status}`;
  } catch {
    return `Request failed: ${response.status}`;
  }
}

export async function sendChatCompletion(
  projectId: string,
  body: ChatCompletionRequest,
  token?: string,
): Promise<ChatCompletionResponse> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/chat/completions`,
    {
      method: "POST",
      headers: authHeaders(token),
      body: JSON.stringify(body),
    },
  );

  if (!response.ok) {
    throw new Error(await parseError(response));
  }

  return response.json();
}

export async function fetchConversations(
  projectId: string,
  token?: string,
): Promise<ChatListResponse<ChatConversation>> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/chat/conversations`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(await parseError(response));
  }

  return response.json();
}

export async function fetchMessages(
  projectId: string,
  conversationId: string,
  token?: string,
): Promise<ChatListResponse<ChatMessageRecord>> {
  const response = await fetch(
    `${API_URL}/api/v1/projects/${projectId}/chat/conversations/${conversationId}/messages`,
    { headers: authHeaders(token) },
  );

  if (!response.ok) {
    throw new Error(await parseError(response));
  }

  return response.json();
}
