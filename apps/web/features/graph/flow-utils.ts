import type { GraphEdge, GraphResponse, NodeType, ServiceType } from "@neuralscope/shared";
import type { Edge, Node } from "@xyflow/react";
import { MarkerType } from "@xyflow/react";

const NODE_TYPE_COLORS: Record<NodeType, string> = {
  browser: "#818cf8",
  service: "#38bdf8",
  database: "#f472b6",
  cache: "#fbbf24",
  queue: "#a78bfa",
  external: "#fb7185",
  unknown: "#71717a",
};

const SERVICE_TYPE_COLORS: Record<ServiceType, string> = {
  frontend: "#818cf8",
  gateway: "#6366f1",
  api: "#38bdf8",
  auth: "#22d3ee",
  database: "#f472b6",
  cache: "#fbbf24",
  queue: "#a78bfa",
  external: "#fb7185",
};

export function nodeColor(
  nodeType?: NodeType,
  serviceType?: ServiceType,
): string {
  if (serviceType) {
    return SERVICE_TYPE_COLORS[serviceType] ?? SERVICE_TYPE_COLORS.api;
  }
  if (nodeType) {
    return NODE_TYPE_COLORS[nodeType] ?? NODE_TYPE_COLORS.unknown;
  }
  return NODE_TYPE_COLORS.unknown;
}

export function toFlowGraph(graph: GraphResponse): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = graph.nodes.map((node) => ({
    id: node.id,
    type: "serviceNode",
    position: node.position,
    data: {
      label: node.label,
      nodeType: node.nodeType,
      serviceType: node.serviceType,
      eventCount: node.data.eventCount,
      totalBytes: node.data.totalBytes,
      color: nodeColor(node.nodeType, node.serviceType),
    },
  }));

  const maxBytes = Math.max(
    1,
    ...graph.edges.map((edge) => edge.data.totalBytes),
  );

  const edges: Edge[] = graph.edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    label: edge.label,
    animated: edge.data.eventCount > 5,
    markerEnd: { type: MarkerType.ArrowClosed, color: "#52525b" },
    style: {
      stroke: "#52525b",
      strokeWidth: edgeStrokeWidth(edge, maxBytes),
    },
    data: edge.data,
  }));

  return { nodes, edges };
}

function edgeStrokeWidth(edge: GraphEdge, maxBytes: number): number {
  if (edge.data.totalBytes <= 0) return 1.5;
  return 1.5 + (edge.data.totalBytes / maxBytes) * 4;
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${bytes} B`;
}
