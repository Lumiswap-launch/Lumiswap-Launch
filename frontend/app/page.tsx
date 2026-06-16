"use client";
import { useState } from "react";
import TokenGrid from "@/components/TokenGrid";
import LaunchWizard from "@/components/LaunchWizard";

export default function Home() {
  const [showWizard, setShowWizard] = useState(false);

  return (
    <main className="max-w-7xl mx-auto px-4 py-12">
      <h1 className="text-4xl font-bold text-center mb-2 text-white">Lumiswap Launch</h1>
      <p className="text-center text-gray-400 mb-6">Permissionless token launches on Stellar with bonding curve price discovery</p>

      <div className="flex justify-center mb-10">
        <button
          onClick={() => setShowWizard((s) => !s)}
          className="px-6 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold transition-colors"
        >
          {showWizard ? "✕ Close" : "🚀 Create Launch"}
        </button>
      </div>

      {showWizard && <LaunchWizard onClose={() => setShowWizard(false)} />}

      <div className={showWizard ? "mt-10" : ""}>
        <TokenGrid />
      </div>
    </main>
  );
}
