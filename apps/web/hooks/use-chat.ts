"use client";

import { useCallback, useEffect, useState } from "react";

import type { ChatConversation, ChatMessageRecord } from "@neuralscope/shared";

import {
  fetchConversations,
  fetchMessages,
  sendChatCompletion,
} from "@/services/chat";

export interface UiChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  toolCallsMade?: number;
  provider?: string;
  pending?: boolean;
}

const TOOL_STATUS_MESSAGES = [
  "Searching logs...",
  "Querying metrics...",
  "Inspecting traces...",
  "Checking deployments...",
  "Analyzing network events...",
];

function toUiMessage(record: ChatMessageRecord): UiChatMessage | null {
  if (record.role !== "user" && record.role !== "assistant") {
    return null;
  }

  return {
    id: record.id,
    role: record.role,
    content: record.content,
    toolCallsMade: record.toolCalls?.toolCallsMade as number | undefined,
  };
}

export function useChat(projectId?: string, token?: string) {
  const [messages, setMessages] = useState<UiChatMessage[]>([]);
  const [conversations, setConversations] = useState<ChatConversation[]>([]);
  const [conversationId, setConversationId] = useState<string>();
  const [loading, setLoading] = useState(false);
  const [loadingHistory, setLoadingHistory] = useState(false);
  const [error, setError] = useState<string>();
  const [toolStatus, setToolStatus] = useState<string>();

  const refreshConversations = useCallback(async () => {
    if (!projectId) return;

    try {
      const response = await fetchConversations(projectId, token);
      setConversations(response.data);
    } catch {
      setConversations([]);
    }
  }, [projectId, token]);

  useEffect(() => {
    refreshConversations();
  }, [refreshConversations]);

  const selectConversation = useCallback(
    async (id: string) => {
      if (!projectId) return;

      setConversationId(id);
      setError(undefined);
      setLoadingHistory(true);

      try {
        const response = await fetchMessages(projectId, id, token);
        setMessages(
          response.data
            .map(toUiMessage)
            .filter((message): message is UiChatMessage => message !== null),
        );
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load messages");
        setMessages([]);
      } finally {
        setLoadingHistory(false);
      }
    },
    [projectId, token],
  );

  const startNewConversation = useCallback(() => {
    setConversationId(undefined);
    setMessages([]);
    setError(undefined);
  }, []);

  const send = useCallback(
    async (message: string) => {
      if (!projectId || !message.trim()) return;

      const trimmed = message.trim();
      const optimisticId = `pending-${Date.now()}`;

      setError(undefined);
      setLoading(true);
      setMessages((current) => [
        ...current,
        { id: optimisticId, role: "user", content: trimmed },
        {
          id: `thinking-${Date.now()}`,
          role: "assistant",
          content: "",
          pending: true,
        },
      ]);

      let statusIndex = 0;
      setToolStatus(TOOL_STATUS_MESSAGES[0]);
      const statusInterval = window.setInterval(() => {
        statusIndex = (statusIndex + 1) % TOOL_STATUS_MESSAGES.length;
        setToolStatus(TOOL_STATUS_MESSAGES[statusIndex]);
      }, 1800);

      try {
        const response = await sendChatCompletion(
          projectId,
          { message: trimmed, conversationId },
          token,
        );

        setConversationId(response.conversationId);
        setMessages((current) => {
          const withoutPending = current.filter((item) => !item.pending);
          return [
            ...withoutPending,
            {
              id: `assistant-${Date.now()}`,
              role: "assistant",
              content: response.content,
              toolCallsMade: response.toolCallsMade,
              provider: response.provider,
            },
          ];
        });

        await refreshConversations();
      } catch (err) {
        setMessages((current) => current.filter((item) => !item.pending));
        setError(err instanceof Error ? err.message : "Failed to send message");
      } finally {
        window.clearInterval(statusInterval);
        setToolStatus(undefined);
        setLoading(false);
      }
    },
    [projectId, token, conversationId, refreshConversations],
  );

  return {
    messages,
    conversations,
    conversationId,
    loading,
    loadingHistory,
    error,
    toolStatus,
    send,
    selectConversation,
    startNewConversation,
    refreshConversations,
  };
}
