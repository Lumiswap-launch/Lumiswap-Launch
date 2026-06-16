"use client";
import { useState } from "react";
import dynamic from "next/dynamic";

const BondingCurveChart = dynamic(() => import("./BondingCurveChart"), { ssr: false });

interface TokenCardProps {
  name: string;
  creator: string;
  xlm_raised: number;
  target_xlm: number;
  price: number;
  total_supply?: number;
}

export default function TokenCard({
  name, creator, xlm_raised, target_xlm, price, total_supply = 1_000_000_000,
}: TokenCardProps) {
  const [showChart, setShowChart] = useState(false);
  const pct = Math.min((xlm_raised / target_xlm) * 100, 100).toFixed(1);
  const short = `${creator.slice(0, 6)}...${creator.slice(-4)}`;
  const sold = (xlm_raised / target_xlm) * total_supply;

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-2xl p-5 flex flex-col gap-3">
      <div className="flex justify-between items-start">
        <h2 className="text-lg font-semibold">{name}</h2>
        <span className="text-xs text-gray-400">{short}</span>
      </div>

      <div>
        <div className="flex justify-between text-xs text-gray-400 mb-1">
          <span>{xlm_raised.toLocaleString()} XLM raised</span>
          <span>{pct}%</span>
        </div>
        <div className="w-full bg-gray-800 rounded-full h-2">
          <div className="bg-indigo-500 h-2 rounded-full" style={{ width: `${pct}%` }} />
        </div>
        <div className="text-xs text-gray-500 mt-1">Target: {target_xlm.toLocaleString()} XLM</div>
      </div>

      <div className="text-sm text-gray-300">
        Price: <span className="text-white font-medium">{price} XLM</span>
      </div>

      <button
        onClick={() => setShowChart((s) => !s)}
        className="text-xs text-indigo-400 hover:text-indigo-300 text-left"
      >
        {showChart ? "▲ Hide chart" : "▼ Show curve"}
      </button>

      {showChart && (
        <div className="h-40">
          <BondingCurveChart
            totalSupply={total_supply}
            targetXlm={target_xlm}
            currentSold={sold}
          />
        </div>
      )}

      <div className="flex gap-2 mt-1">
        <button className="flex-1 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-sm font-medium transition-colors">Buy</button>
        <button className="flex-1 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-sm font-medium transition-colors">Sell</button>
      </div>
    </div>
  );
}
