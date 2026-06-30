"use client";

import { Bot, User, Wrench } from "lucide-react";

import { cn } from "@/lib/utils";

import type { UiChatMessage } from "@/hooks/use-chat";

interface MessageBubbleProps {
  message: UiChatMessage;
  toolStatus?: string;
}

export function MessageBubble({ message, toolStatus }: MessageBubbleProps) {
  const isUser = message.role === "user";

  if (message.pending) {
    return (
      <div className="flex gap-3">
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-indigo-600/20">
          <Bot className="h-4 w-4 text-indigo-400" />
        </div>
        <div className="glass max-w-[85%] rounded-2xl rounded-tl-sm px-4 py-3">
          <div className="flex items-center gap-2 text-sm text-zinc-400">
            <span className="inline-flex gap-1">
              <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-indigo-400 [animation-delay:0ms]" />
              <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-indigo-400 [animation-delay:150ms]" />
              <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-indigo-400 [animation-delay:300ms]" />
            </span>
            {toolStatus ?? "Thinking..."}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={cn("flex gap-3", isUser && "flex-row-reverse")}>
      <div
        className={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center rounded-lg",
          isUser ? "bg-zinc-800" : "bg-indigo-600/20",
        )}
      >
        {isUser ? (
          <User className="h-4 w-4 text-zinc-400" />
        ) : (
          <Bot className="h-4 w-4 text-indigo-400" />
        )}
      </div>

      <div
        className={cn(
          "max-w-[85%] rounded-2xl px-4 py-3 text-sm leading-relaxed",
          isUser
            ? "rounded-tr-sm bg-indigo-600 text-white"
            : "glass rounded-tl-sm text-zinc-200",
        )}
      >
        <p className="whitespace-pre-wrap">{message.content}</p>

        {!isUser && (message.toolCallsMade !== undefined || message.provider) && (
          <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-zinc-800/60 pt-2 text-xs text-zinc-500">
            {message.toolCallsMade !== undefined && message.toolCallsMade > 0 && (
              <span className="inline-flex items-center gap-1 rounded-full bg-zinc-800/80 px-2 py-0.5 text-zinc-400">
                <Wrench className="h-3 w-3" />
                {message.toolCallsMade} tool
                {message.toolCallsMade === 1 ? "" : "s"} used
              </span>
            )}
            {message.provider && (
              <span className="rounded-full bg-zinc-800/80 px-2 py-0.5 capitalize">
                {message.provider}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
