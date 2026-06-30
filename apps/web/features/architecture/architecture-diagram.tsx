"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { RefreshCw, Sparkles } from "lucide-react";

import { flowNodeTypes } from "@/features/graph/service-node";
import { toFlowGraph } from "@/features/graph/flow-utils";
import { cn } from "@/lib/utils";
import {
  fetchArchitectureGraph,
  regenerateArchitectureGraph,
} from "@/services/architecture";
import { ingestSampleNetworkEvents } from "@/services/network";

interface ArchitectureDiagramProps {
  projectId?: string;
  token?: string;
}

export function ArchitectureDiagram({
  projectId,
  token,
}: ArchitectureDiagramProps) {
  const [nodes, setNodes] = useState<ReturnType<typeof toFlowGraph>["nodes"]>([]);
  const [edges, setEdges] = useState<ReturnType<typeof toFlowGraph>["edges"]>([]);
  const [loading, setLoading] = useState(true);
  const [regenerating, setRegenerating] = useState(false);
  const [error, setError] = useState<string>();

  const loadGraph = useCallback(async () => {
    if (!projectId) return;

    setLoading(true);
    setError(undefined);

    try {
      const graph = await fetchArchitectureGraph(projectId, token);
      const flow = toFlowGraph(graph);
      setNodes(flow.nodes);
      setEdges(flow.edges);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load architecture");
    } finally {
      setLoading(false);
    }
  }, [projectId, token]);

  useEffect(() => {
    void loadGraph();
  }, [loadGraph]);

  async function handleRegenerate(withSample = false) {
    if (!projectId) return;

    setRegenerating(true);
    setError(undefined);

    try {
      if (withSample) {
        await ingestSampleNetworkEvents(projectId, token);
      }
      const graph = await regenerateArchitectureGraph(projectId, token);
      const flow = toFlowGraph(graph);
      setNodes(flow.nodes);
      setEdges(flow.edges);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to regenerate graph");
    } finally {
      setRegenerating(false);
    }
  }

  const isEmpty = useMemo(
    () => !loading && nodes.length === 0,
    [loading, nodes.length],
  );

  if (!projectId) {
    return (
      <div className="glass flex h-[calc(100vh-8rem)] items-center justify-center rounded-xl text-sm text-zinc-500">
        Sign in and create a project to view service architecture.
      </div>
    );
  }

  return (
    <div className="glass relative h-[calc(100vh-8rem)] overflow-hidden rounded-xl">
      <div className="absolute right-4 top-4 z-10 flex gap-2">
        <button
          type="button"
          onClick={() => void loadGraph()}
          disabled={loading || regenerating}
          className="flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900/90 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800 disabled:opacity-50"
        >
          <RefreshCw
            className={cn("h-3.5 w-3.5", (loading || regenerating) && "animate-spin")}
          />
          Refresh
        </button>
        <button
          type="button"
          onClick={() => void handleRegenerate(isEmpty)}
          disabled={regenerating}
          className="flex items-center gap-2 rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-indigo-500 disabled:opacity-50"
        >
          <Sparkles className="h-3.5 w-3.5" />
          {isEmpty ? "Generate from sample data" : "Regenerate"}
        </button>
      </div>

      {loading && nodes.length === 0 ? (
        <div className="flex h-full items-center justify-center text-sm text-zinc-500">
          Loading architecture diagram...
        </div>
      ) : isEmpty ? (
        <div className="flex h-full flex-col items-center justify-center gap-3 px-8 text-center">
          <Sparkles className="h-8 w-8 text-indigo-400" />
          <p className="text-sm text-zinc-400">
            No architecture graph yet. Generate one from network events and trace
            dependencies.
          </p>
          <p className="text-xs text-zinc-600">
            Use &quot;Generate from sample data&quot; to bootstrap a demo topology.
          </p>
        </div>
      ) : (
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={flowNodeTypes}
          fitView
          minZoom={0.25}
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
  );
}
