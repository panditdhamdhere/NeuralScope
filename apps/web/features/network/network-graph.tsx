"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { RefreshCw } from "lucide-react";

import { flowNodeTypes } from "@/features/graph/service-node";
import { formatBytes, toFlowGraph } from "@/features/graph/flow-utils";
import { cn } from "@/lib/utils";
import {
  fetchNetworkEvents,
  fetchNetworkGraph,
  ingestSampleNetworkEvents,
} from "@/services/network";

interface NetworkGraphProps {
  projectId?: string;
  token?: string;
}

export function NetworkGraph({ projectId, token }: NetworkGraphProps) {
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<ReturnType<typeof toFlowGraph>["edges"]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const [recentEvents, setRecentEvents] = useState<
    Awaited<ReturnType<typeof fetchNetworkEvents>>["data"]
  >([]);

  const loadGraph = useCallback(async () => {
    if (!projectId) return;

    setLoading(true);
    setError(undefined);

    try {
      const graph = await fetchNetworkGraph(projectId, token);
      const flow = toFlowGraph(graph);
      setNodes(flow.nodes);
      setEdges(flow.edges);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load graph");
    } finally {
      setLoading(false);
    }
  }, [projectId, token]);

  useEffect(() => {
    void loadGraph();
  }, [loadGraph]);

  useEffect(() => {
    if (!projectId || !selectedNode) {
      setRecentEvents([]);
      return;
    }

    fetchNetworkEvents(
      projectId,
      { source: selectedNode.data.label as string, limit: 10 },
      token,
    )
      .then((response) => setRecentEvents(response.data))
      .catch(() => setRecentEvents([]));
  }, [projectId, token, selectedNode]);

  async function handleLoadSample() {
    if (!projectId) return;
    setLoading(true);
    try {
      await ingestSampleNetworkEvents(projectId, token);
      await loadGraph();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load sample data");
      setLoading(false);
    }
  }

  const isEmpty = useMemo(
    () => !loading && nodes.length === 0,
    [loading, nodes.length],
  );

  if (!projectId) {
    return (
      <div className="glass flex h-[calc(100vh-8rem)] items-center justify-center rounded-xl text-sm text-zinc-500">
        Sign in and create a project to view network traffic.
      </div>
    );
  }

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-4">
      <div className="glass relative min-w-0 flex-1 overflow-hidden rounded-xl">
        <div className="absolute right-4 top-4 z-10 flex gap-2">
          <button
            type="button"
            onClick={() => void loadGraph()}
            className="flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900/90 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
          >
            <RefreshCw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
            Refresh
          </button>
          {isEmpty && (
            <button
              type="button"
              onClick={() => void handleLoadSample()}
              className="rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-indigo-500"
            >
              Load sample data
            </button>
          )}
        </div>

        {loading && nodes.length === 0 ? (
          <div className="flex h-full items-center justify-center text-sm text-zinc-500">
            Loading network graph...
          </div>
        ) : isEmpty ? (
          <div className="flex h-full flex-col items-center justify-center gap-3 text-center text-sm text-zinc-500">
            <p>No network events yet.</p>
            <p className="text-xs text-zinc-600">
              Ingest events via the API or load sample data to visualize connections.
            </p>
          </div>
        ) : (
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={flowNodeTypes}
            onNodeClick={(_, node) => setSelectedNode(node)}
            fitView
            minZoom={0.3}
            maxZoom={1.5}
            proOptions={{ hideAttribution: true }}
          >
            <Background color="#27272a" gap={20} />
            <Controls className="!bg-zinc-900 !border-zinc-700" />
            <MiniMap
              className="!bg-zinc-900 !border-zinc-700"
              nodeColor={(node) => (node.data.color as string) ?? "#71717a"}
            />
          </ReactFlow>
        )}

        {error && (
          <div className="absolute bottom-4 left-4 right-4 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-300">
            {error}
          </div>
        )}
      </div>

      <aside className="glass w-72 shrink-0 rounded-xl p-4">
        <h3 className="text-sm font-medium text-white">Node details</h3>
        {selectedNode ? (
          <div className="mt-4 space-y-3 text-sm">
            <div>
              <p className="text-xs text-zinc-500">Service</p>
              <p className="font-medium text-zinc-200">
                {selectedNode.data.label as string}
              </p>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div className="rounded-lg bg-zinc-900/60 p-2">
                <p className="text-zinc-500">Events</p>
                <p className="text-zinc-200">{selectedNode.data.eventCount as number}</p>
              </div>
              <div className="rounded-lg bg-zinc-900/60 p-2">
                <p className="text-zinc-500">Traffic</p>
                <p className="text-zinc-200">
                  {formatBytes(selectedNode.data.totalBytes as number)}
                </p>
              </div>
            </div>
            <div>
              <p className="mb-2 text-xs text-zinc-500">Recent outbound events</p>
              <div className="max-h-64 space-y-2 overflow-y-auto">
                {recentEvents.length === 0 ? (
                  <p className="text-xs text-zinc-600">No recent events</p>
                ) : (
                  recentEvents.map((event) => (
                    <div
                      key={event.id}
                      className="rounded-lg border border-zinc-800 bg-zinc-900/50 p-2 text-xs"
                    >
                      <p className="text-zinc-300">
                        → {event.destination.name}
                      </p>
                      <p className="mt-1 text-zinc-600">
                        {event.protocol}
                        {event.latencyMs != null && ` · ${event.latencyMs.toFixed(0)}ms`}
                      </p>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        ) : (
          <p className="mt-4 text-xs text-zinc-600">
            Click a node to inspect traffic and recent connections.
          </p>
        )}
      </aside>
    </div>
  );
}
