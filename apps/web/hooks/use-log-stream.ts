"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import type { LogLevel } from "@neuralscope/shared";

import { buildLogWebSocketUrl } from "@/services/logs";

export interface LiveLogEvent {
  id: string;
  level: LogLevel;
  message: string;
  timestamp: string;
}

interface UseLogStreamOptions {
  projectId?: string;
  token?: string;
  enabled?: boolean;
  maxEntries?: number;
}

export function useLogStream({
  projectId,
  token,
  enabled = true,
  maxEntries = 200,
}: UseLogStreamOptions) {
  const [logs, setLogs] = useState<LiveLogEvent[]>([]);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const clear = useCallback(() => setLogs([]), []);

  useEffect(() => {
    if (!enabled || !projectId) {
      return;
    }

    const url = buildLogWebSocketUrl(projectId, token);
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnected(true);
      setError(null);
    };

    ws.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data as string) as {
          type?: string;
          event?: {
            type?: string;
            payload?: { entry_id?: string; level?: string; message?: string };
          };
          timestamp?: string;
        };

        if (payload.type === "warning") {
          return;
        }

        if (payload.event?.type === "log.new" && payload.event.payload) {
          const { entry_id, level, message } = payload.event.payload;
          if (!entry_id || !level || !message) return;

          setLogs((prev) => {
            const next: LiveLogEvent[] = [
              {
                id: entry_id,
                level: level as LogLevel,
                message,
                timestamp: payload.timestamp ?? new Date().toISOString(),
              },
              ...prev,
            ];
            return next.slice(0, maxEntries);
          });
        }
      } catch {
        setError("Received malformed log event");
      }
    };

    ws.onerror = () => {
      setError("WebSocket connection error");
      setConnected(false);
    };

    ws.onclose = () => {
      setConnected(false);
    };

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [enabled, projectId, token, maxEntries]);

  return { logs, connected, error, clear };
}
