import Link from 'next/link';
import { ArrowRight, Download, Play, Users, Zap } from 'lucide-react';

export default function Home() {
  return (
    <div className="min-h-screen bg-slate-950 text-white selection:bg-indigo-500/30">
      {/* Navigation */}
      <nav className="fixed top-0 w-full z-50 border-b border-white/5 bg-slate-950/80 backdrop-blur-xl">
        <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2 font-bold text-xl tracking-tight">
            <div className="w-8 h-8 rounded-lg bg-indigo-600 flex items-center justify-center">
              <Play size={16} className="fill-white" />
            </div>
            FluxReplay
          </div>
          <div className="flex items-center gap-6 text-sm font-medium text-slate-400">
            <Link href="#features" className="hover:text-white transition-colors">
              Features
            </Link>
            <Link href="#download" className="hover:text-white transition-colors">
              Download
            </Link>
            <Link
              href="/login"
              className="px-4 py-2 rounded-full bg-white/5 hover:bg-white/10 text-white transition-colors"
            >
              Login
            </Link>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-7xl mx-auto text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 text-xs font-medium mb-8">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-indigo-500"></span>
            </span>
            Now in Beta
          </div>

          <h1 className="text-5xl md:text-7xl font-bold tracking-tight mb-8 bg-gradient-to-b from-white to-slate-400 bg-clip-text text-transparent">
            Sync Your Squad.
            <br />
            Replay the Glory.
          </h1>

          <p className="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto mb-12 leading-relaxed">
            The first replay tool designed for teams. Automatically syncs recordings from everyone
            in your party, so you can watch every play from every angle.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <button className="px-8 py-4 rounded-2xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-lg transition-all hover:scale-105 active:scale-95 flex items-center gap-3 shadow-[0_0_40px_-10px_rgba(79,70,229,0.5)]">
              <Download size={24} />
              Download for Windows
            </button>
            <button className="px-8 py-4 rounded-2xl bg-white/5 hover:bg-white/10 text-white font-bold text-lg transition-all border border-white/10 flex items-center gap-3">
              View Demo Room
              <ArrowRight size={20} />
            </button>
          </div>

          {/* Feature Grid */}
          <div className="grid md:grid-cols-3 gap-8 mt-32 text-left">
            <div className="p-8 rounded-3xl bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
              <div className="w-12 h-12 rounded-2xl bg-emerald-500/20 flex items-center justify-center text-emerald-400 mb-6">
                <Users size={24} />
              </div>
              <h3 className="text-xl font-bold mb-3">Squad Sync</h3>
              <p className="text-slate-400 leading-relaxed">
                Connect with your team. When one person clips, everyone clips. Automatically.
              </p>
            </div>
            <div className="p-8 rounded-3xl bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
              <div className="w-12 h-12 rounded-2xl bg-amber-500/20 flex items-center justify-center text-amber-400 mb-6">
                <Zap size={24} />
              </div>
              <h3 className="text-xl font-bold mb-3">Instant Replay</h3>
              <p className="text-slate-400 leading-relaxed">
                Zero-performance-impact buffering. Save the last 60 seconds without dropping a
                frame.
              </p>
            </div>
            <div className="p-8 rounded-3xl bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
              <div className="w-12 h-12 rounded-2xl bg-rose-500/20 flex items-center justify-center text-rose-400 mb-6">
                <Play size={24} />
              </div>
              <h3 className="text-xl font-bold mb-3">Multi-View Playback</h3>
              <p className="text-slate-400 leading-relaxed">
                Watch the action from every perspective. Frame-perfect synchronization across all
                users.
              </p>
            </div>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-white/5 py-12 text-center text-slate-500 text-sm">
        <p>&copy; {new Date().getFullYear()} FluxReplay. All rights reserved.</p>
      </footer>
    </div>
  );
}
