"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Activity,
  AlertTriangle,
  Bot,
  GitBranch,
  LayoutDashboard,
  Network,
  ScrollText,
  Settings,
  Shield,
  Waypoints,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useProjectSession } from "@/hooks/use-project-session";

const navigation = [
  { name: "Overview", href: "/overview", icon: LayoutDashboard },
  { name: "Live Logs", href: "/logs", icon: ScrollText },
  { name: "Metrics", href: "/metrics", icon: Activity },
  { name: "Traces", href: "/traces", icon: Waypoints },
  { name: "AI Chat", href: "/chat", icon: Bot },
  { name: "Git History", href: "/git", icon: GitBranch },
  { name: "Architecture", href: "/architecture", icon: Network },
  { name: "Network", href: "/network", icon: Network },
  { name: "Security", href: "/security", icon: Shield },
  { name: "Incidents", href: "/incidents", icon: AlertTriangle },
  { name: "Settings", href: "/settings", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();
  const { project, loading } = useProjectSession();

  return (
    <aside className="flex h-screen w-60 flex-col border-r border-zinc-800/50 bg-zinc-950/80 backdrop-blur-xl">
      <div className="flex h-14 items-center gap-2 border-b border-zinc-800/50 px-4">
        <div className="flex h-7 w-7 items-center justify-center rounded-md bg-indigo-600">
          <Activity className="h-4 w-4 text-white" />
        </div>
        <span className="font-semibold text-white">
          Neural<span className="text-indigo-400">Scope</span>
        </span>
      </div>

      <nav className="flex-1 space-y-0.5 overflow-y-auto p-3">
        {navigation.map((item) => {
          const isActive = pathname.startsWith(item.href);
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-all duration-150",
                isActive
                  ? "bg-indigo-600/15 text-indigo-300"
                  : "text-zinc-400 hover:bg-zinc-800/50 hover:text-white",
              )}
            >
              <item.icon className="h-4 w-4 shrink-0" />
              {item.name}
            </Link>
          );
        })}
      </nav>

      <div className="border-t border-zinc-800/50 p-3">
        <div className="glass rounded-lg px-3 py-2">
          <p className="text-xs text-zinc-500">Project</p>
          <p className="truncate text-sm font-medium text-zinc-300">
            {loading ? "Loading..." : (project?.name ?? "No project")}
          </p>
        </div>
      </div>
    </aside>
  );
}
