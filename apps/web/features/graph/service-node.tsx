"use client";

import { Handle, Position, type NodeProps } from "@xyflow/react";

import { formatBytes } from "@/features/graph/flow-utils";

export function ServiceFlowNode({ data }: NodeProps) {
  const label = String(data.label ?? "");
  const color = String(data.color ?? "#71717a");
  const eventCount = Number(data.eventCount ?? 0);
  const totalBytes = Number(data.totalBytes ?? 0);
  const typeLabel = String(data.serviceType ?? data.nodeType ?? "service");

  return (
    <div
      className="min-w-[140px] rounded-xl border border-zinc-700/80 bg-zinc-900/90 px-4 py-3 shadow-lg backdrop-blur-sm"
      style={{ borderTopColor: color, borderTopWidth: 3 }}
    >
      <Handle type="target" position={Position.Top} className="!bg-zinc-500" />
      <p className="text-sm font-medium text-white">{label}</p>
      <p className="mt-1 text-xs capitalize text-zinc-500">{typeLabel}</p>
      <p className="mt-2 text-[11px] text-zinc-600">
        {eventCount} events · {formatBytes(totalBytes)}
      </p>
      <Handle type="source" position={Position.Bottom} className="!bg-zinc-500" />
    </div>
  );
}

export const flowNodeTypes = {
  serviceNode: ServiceFlowNode,
};
