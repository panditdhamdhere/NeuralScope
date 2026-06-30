/** Shared TypeScript types used across frontend and API contracts. */

export type LogLevel = "trace" | "debug" | "info" | "warn" | "error" | "fatal";

export interface LogEntry {
  id: string;
  projectId: string;
  timestamp: string;
  level: LogLevel;
  message: string;
  service?: string;
  traceId?: string;
  metadata: Record<string, unknown>;
}

export interface MetricPoint {
  id: string;
  projectId: string;
  name: string;
  value: number;
  unit: "count" | "percent" | "bytes" | "milliseconds" | "requests_per_second";
  tags: Record<string, string>;
  timestamp: string;
}

export type CheckStatus = "up" | "down";
export type ReadinessStatus = "ready" | "not_ready";

export interface DependencyCheck {
  status: CheckStatus;
  latency_ms?: number;
  error?: string;
}

export interface LivenessResponse {
  status: "ok";
  version: string;
}

export interface ReadinessResponse {
  status: ReadinessStatus;
  version: string;
  checks: {
    database: DependencyCheck;
    redis: DependencyCheck;
  };
}

export interface StatusResponse extends ReadinessResponse {
  environment: string;
  uptime_seconds: number;
}

export interface ApiError {
  error: {
    message: string;
    code: number;
  };
}

export interface User {
  id: string;
  email: string;
  name?: string;
  emailVerified: boolean;
  image?: string;
  createdAt: string;
}

export interface Project {
  id: string;
  name: string;
  slug: string;
  ownerId: string;
  role: "owner" | "admin" | "viewer";
  createdAt: string;
}

export interface ApiKey {
  id: string;
  userId: string;
  name: string;
  keyPrefix: string;
  lastUsedAt?: string;
  expiresAt?: string;
  createdAt: string;
}

export type TraceStatus = "ok" | "error" | "unset";

export interface Trace {
  id: string;
  projectId: string;
  traceId: string;
  rootService: string;
  durationMs: number;
  spanCount: number;
  status: TraceStatus;
  startedAt: string;
}

export interface Span {
  id: string;
  traceId: string;
  spanId: string;
  parentSpanId?: string;
  service: string;
  operation: string;
  durationMs: number;
  status: TraceStatus;
  attributes: Record<string, unknown>;
  startedAt: string;
}

export interface ChatCompletionRequest {
  message: string;
  conversationId?: string;
}

export interface ChatCompletionResponse {
  conversationId: string;
  content: string;
  toolCallsMade: number;
  provider: string;
}

export interface ChatConversation {
  id: string;
  title?: string;
  createdAt: string;
  updatedAt: string;
}

export interface ChatMessageRecord {
  id: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  toolCalls?: {
    toolCallsMade?: number;
    [key: string]: unknown;
  };
  createdAt: string;
}

export type NodeType =
  | "service"
  | "external"
  | "database"
  | "cache"
  | "queue"
  | "browser"
  | "unknown";

export type ServiceType =
  | "gateway"
  | "api"
  | "auth"
  | "database"
  | "cache"
  | "queue"
  | "external"
  | "frontend";

export interface GraphNode {
  id: string;
  label: string;
  nodeType?: NodeType;
  serviceType?: ServiceType;
  position: { x: number; y: number };
  data: {
    eventCount: number;
    totalBytes: number;
  };
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  data: {
    protocol: string;
    eventCount: number;
    totalBytes: number;
    avgLatencyMs?: number;
  };
}

export interface GraphResponse {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface NetworkEvent {
  id: string;
  projectId: string;
  source: { name: string; nodeType: NodeType };
  destination: { name: string; nodeType: NodeType };
  protocol: string;
  bytesSent: number;
  bytesReceived: number;
  latencyMs?: number;
  timestamp: string;
}

export type Severity = "low" | "medium" | "high" | "critical";

export type FindingType =
  | "secret"
  | "api_key"
  | "exposed_port"
  | "weak_config"
  | "dependency_vulnerability"
  | "docker_issue";

export interface SecurityFinding {
  id: string;
  projectId: string;
  findingType: FindingType;
  severity: Severity;
  title: string;
  description: string;
  resource?: string;
  detectedAt: string;
}

export interface ScanResult {
  findings: SecurityFinding[];
  scannedSources: number;
}

export type IncidentStatus = "open" | "investigating" | "resolved";

export interface TimelineEntry {
  timestamp: string;
  entryType: "log" | "trace" | "finding" | "metric" | "system";
  title: string;
  detail: string;
}

export interface Incident {
  id: string;
  projectId: string;
  title: string;
  severity: Severity;
  status: IncidentStatus;
  rootCause?: string;
  timeline: TimelineEntry[];
  affectedServices: string[];
  suggestedFixes: string[];
  createdAt: string;
  resolvedAt?: string;
}

export interface ProjectOverview {
  errorLogs24h: number;
  totalLogs24h: number;
  traces24h: number;
  failedTraces24h: number;
  openIncidents: number;
  criticalFindings: number;
  conversations: number;
  recentLogs: {
    id: string;
    level: string;
    message: string;
    service?: string;
    timestamp: string;
  }[];
  cpuUsage?: number;
  memoryUsage?: number;
  serverStatus: string;
}

