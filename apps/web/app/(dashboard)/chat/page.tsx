"use client";

import { ChatPanel } from "@/features/chat/chat-panel";
import { useProjectSession } from "@/hooks/use-project-session";

export default function ChatPage() {
  const { projectId, token, loading } = useProjectSession();

  return (
    <div className="p-8">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">AI Chat</h1>
        <p className="mt-1 text-sm text-zinc-400">
          Ask questions about your application — NeuralScope retrieves evidence
          from logs, metrics, traces, and more.
        </p>
      </div>

      {loading ? (
        <div className="glass flex h-[calc(100vh-8rem)] items-center justify-center rounded-xl text-sm text-zinc-500">
          Loading...
        </div>
      ) : (
        <ChatPanel projectId={projectId} token={token} />
      )}
    </div>
  );
}
