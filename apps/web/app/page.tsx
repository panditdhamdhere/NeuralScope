import Link from "next/link";

export default function HomePage() {
  return (
    <main className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden">
      {/* Background gradient */}
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/2 top-0 h-[600px] w-[600px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-indigo-600/10 blur-3xl" />
        <div className="absolute bottom-0 right-0 h-[400px] w-[400px] translate-x-1/4 translate-y-1/4 rounded-full bg-violet-600/10 blur-3xl" />
      </div>

      <div className="relative z-10 flex max-w-2xl flex-col items-center gap-8 px-6 text-center">
        <div className="flex items-center gap-2 rounded-full border border-zinc-800 bg-zinc-900/50 px-4 py-1.5 text-sm text-zinc-400">
          <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
          AI-Powered Observability
        </div>

        <h1 className="text-5xl font-bold tracking-tight text-white sm:text-6xl">
          Neural<span className="text-indigo-400">Scope</span>
        </h1>

        <p className="text-lg text-zinc-400 leading-relaxed">
          Ask questions about your application in natural language.
          NeuralScope gathers context from logs, metrics, traces, and Git
          before answering.
        </p>

        <div className="flex gap-4">
          <Link
            href="/login"
            className="rounded-lg bg-indigo-600 px-6 py-2.5 text-sm font-medium text-white transition-colors hover:bg-indigo-500"
          >
            Open Dashboard
          </Link>
          <a
            href="https://github.com/panditdhamdhere/NeuralScope"
            className="rounded-lg border border-zinc-700 px-6 py-2.5 text-sm font-medium text-zinc-300 transition-colors hover:border-zinc-600 hover:text-white"
          >
            View on GitHub
          </a>
        </div>
      </div>
    </main>
  );
}
