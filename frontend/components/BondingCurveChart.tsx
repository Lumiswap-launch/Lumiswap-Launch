"use client";
import {
  Chart as ChartJS,
  LineElement,
  PointElement,
  LinearScale,
  CategoryScale,
  Tooltip,
  Legend,
  type ChartOptions,
} from "chart.js";
import annotationPlugin from "chartjs-plugin-annotation";
import { Line } from "react-chartjs-2";

ChartJS.register(LineElement, PointElement, LinearScale, CategoryScale, Tooltip, Legend, annotationPlugin);

interface Props {
  totalSupply: number;
  targetXlm: number;
  currentSold: number;
}

const VIRTUAL_XLM_INITIAL = 30_000;
const STEPS = 100;

export default function BondingCurveChart({ totalSupply, targetXlm, currentSold }: Props) {
  const labels: string[] = [];
  const prices: number[] = [];

  for (let i = 0; i <= STEPS; i++) {
    const sold = (totalSupply * i) / STEPS;
    // Approximate spot price using constant-product:
    // virtual_xlm grows proportionally with XLM raised
    const virtualXlm = VIRTUAL_XLM_INITIAL + (sold / totalSupply) * targetXlm;
    const virtualTokens = totalSupply - sold || 1;
    prices.push(virtualXlm / virtualTokens);
    labels.push(((sold / totalSupply) * 100).toFixed(0) + "%");
  }

  const currentStep = Math.round((currentSold / totalSupply) * STEPS);

  const data = {
    labels,
    datasets: [
      {
        label: "Price (XLM)",
        data: prices,
        borderColor: "#6366f1",
        backgroundColor: "rgba(99,102,241,0.1)",
        borderWidth: 2,
        pointRadius: 0,
        fill: true,
        tension: 0.4,
      },
    ],
  };

  const options: ChartOptions<"line"> = {
    responsive: true,
    plugins: {
      legend: { display: false },
      // @ts-expect-error annotation plugin types
      annotation: {
        annotations: {
          currentLine: {
            type: "line",
            xMin: currentStep,
            xMax: currentStep,
            borderColor: "rgba(239,68,68,0.8)",
            borderWidth: 2,
            borderDash: [4, 4],
            label: { content: "Current", display: true, color: "#ef4444", font: { size: 11 } },
          },
        },
      },
    },
    scales: {
      x: {
        title: { display: true, text: "% Sold", color: "#9ca3af" },
        ticks: { color: "#6b7280", maxTicksLimit: 6 },
        grid: { color: "rgba(75,85,99,0.3)" },
      },
      y: {
        title: { display: true, text: "Price (XLM)", color: "#9ca3af" },
        ticks: { color: "#6b7280" },
        grid: { color: "rgba(75,85,99,0.3)" },
      },
    },
  };

  return <Line data={data} options={options} />;
}
