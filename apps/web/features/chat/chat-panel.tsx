"use client";

import { FormEvent, useEffect, useRef, useState } from "react";
import { MessageSquarePlus, Send, Sparkles } from "lucide-react";

import type { ChatConversation } from "@neuralscope/shared";

import { useChat } from "@/hooks/use-chat";
import { cn } from "@/lib/utils";

import { MessageBubble } from "./message-bubble";

const SUGGESTED_PROMPTS = [
  "What errors happened recently?",
  "Show me CPU and memory metrics",
  "Are there any slow or failed traces?",
  "What was the last deployment?",
];

interface ChatPanelProps {
  projectId?: string;
  token?: string;
}

function formatRelativeTime(value: string): string {
  const date = new Date(value);
  const diffMs = Date.now() - date.getTime();
  const diffMinutes = Math.floor(diffMs / 60_000);

  if (diffMinutes < 1) return "Just now";
  if (diffMinutes < 60) return `${diffMinutes}m ago`;

  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) return `${diffHours}h ago`;

  return date.toLocaleDateString();
}

function ConversationItem({
  conversation,
  active,
  onSelect,
}: {
  conversation: ChatConversation;
  active: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "w-full rounded-lg px-3 py-2 text-left transition-colors",
        active
          ? "bg-indigo-600/15 text-indigo-200"
          : "text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200",
      )}
    >
      <p className="truncate text-sm font-medium">
        {conversation.title ?? "Untitled conversation"}
      </p>
      <p className="mt-0.5 text-xs text-zinc-600">
        {formatRelativeTime(conversation.updatedAt)}
      </p>
    </button>
  );
}

export function ChatPanel({ projectId, token }: ChatPanelProps) {
  const [input, setInput] = useState("");
  const bottomRef = useRef<HTMLDivElement>(null);

  const {
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
  } = useChat(projectId, token);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, toolStatus]);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (!input.trim() || loading) return;

    const message = input;
    setInput("");
    await send(message);
  }

  function handleSuggestedPrompt(prompt: string) {
    if (loading) return;
    void send(prompt);
  }

  if (!projectId) {
    return (
      <div className="glass flex h-full items-center justify-center rounded-xl p-8 text-center text-sm text-zinc-500">
        Sign in and create a project to start chatting with NeuralScope AI.
      </div>
    );
  }

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-4">
      <aside className="glass flex w-64 shrink-0 flex-col rounded-xl p-3">
        <button
          type="button"
          onClick={startNewConversation}
          className="mb-3 flex items-center justify-center gap-2 rounded-lg bg-indigo-600 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-500"
        >
          <MessageSquarePlus className="h-4 w-4" />
          New chat
        </button>

        <div className="flex-1 space-y-1 overflow-y-auto">
          {conversations.length === 0 ? (
            <p className="px-2 py-4 text-center text-xs text-zinc-600">
              No conversations yet
            </p>
          ) : (
            conversations.map((conversation) => (
              <ConversationItem
                key={conversation.id}
                conversation={conversation}
                active={conversation.id === conversationId}
                onSelect={() => void selectConversation(conversation.id)}
              />
            ))
          )}
        </div>
      </aside>

      <div className="glass flex min-w-0 flex-1 flex-col rounded-xl">
        <div className="flex-1 overflow-y-auto p-4">
          {loadingHistory ? (
            <div className="flex h-full items-center justify-center text-sm text-zinc-500">
              Loading conversation...
            </div>
          ) : messages.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center px-6 text-center">
              <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-2xl bg-indigo-600/15">
                <Sparkles className="h-6 w-6 text-indigo-400" />
              </div>
              <h2 className="text-lg font-medium text-white">
                Ask about your application
              </h2>
              <p className="mt-2 max-w-md text-sm text-zinc-400">
                NeuralScope searches your logs, metrics, traces, deployments, and
                network data before answering.
              </p>

              <div className="mt-6 grid w-full max-w-2xl gap-2 sm:grid-cols-2">
                {SUGGESTED_PROMPTS.map((prompt) => (
                  <button
                    key={prompt}
                    type="button"
                    disabled={loading}
                    onClick={() => handleSuggestedPrompt(prompt)}
                    className="rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3 text-left text-sm text-zinc-300 transition-colors hover:border-indigo-500/40 hover:bg-indigo-600/10 disabled:opacity-50"
                  >
                    {prompt}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            <div className="mx-auto flex max-w-3xl flex-col gap-4">
              {messages.map((message) => (
                <MessageBubble
                  key={message.id}
                  message={message}
                  toolStatus={message.pending ? toolStatus : undefined}
                />
              ))}
              <div ref={bottomRef} />
            </div>
          )}
        </div>

        {error && (
          <div className="border-t border-red-500/20 bg-red-500/10 px-4 py-2 text-sm text-red-300">
            {error}
          </div>
        )}

        <form
          onSubmit={handleSubmit}
          className="border-t border-zinc-800/50 p-4"
        >
          <div className="mx-auto flex max-w-3xl items-end gap-2">
            <textarea
              value={input}
              onChange={(event) => setInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  void handleSubmit(event);
                }
              }}
              rows={1}
              placeholder="Ask about logs, metrics, traces, deployments..."
              disabled={loading}
              className="max-h-32 min-h-[44px] flex-1 resize-none rounded-xl border border-zinc-800 bg-zinc-900/80 px-4 py-3 text-sm text-white placeholder:text-zinc-600 focus:border-indigo-500/50 focus:outline-none disabled:opacity-50"
            />
            <button
              type="submit"
              disabled={loading || !input.trim()}
              className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-indigo-600 text-white transition-colors hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-50"
            >
              <Send className="h-4 w-4" />
            </button>
          </div>
          <p className="mx-auto mt-2 max-w-3xl text-xs text-zinc-600">
            Enter to send · Shift+Enter for new line · AI uses observability tools
            to retrieve real data
          </p>
        </form>
      </div>
    </div>
  );
}
