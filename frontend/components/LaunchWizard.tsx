"use client";
import { useState } from "react";

interface FormState {
  name: string;
  symbol: string;
  totalSupply: string;
  targetXlm: string;
  virtualXlm: string;
}

const INITIAL: FormState = { name: "", symbol: "", totalSupply: "", targetXlm: "", virtualXlm: "30000" };

const STEPS = ["Token Details", "Curve Params", "Review & Deploy"];

export default function LaunchWizard({ onClose }: { onClose: () => void }) {
  const [step, setStep] = useState(1);
  const [form, setForm] = useState<FormState>(INITIAL);
  const [errors, setErrors] = useState<Partial<FormState>>({});

  const set = (k: keyof FormState) => (e: React.ChangeEvent<HTMLInputElement>) =>
    setForm((f) => ({ ...f, [k]: e.target.value }));

  const validateStep1 = () => {
    const e: Partial<FormState> = {};
    if (!form.name.trim()) e.name = "Required";
    if (!form.symbol.trim()) e.symbol = "Required";
    if (!form.totalSupply || Number(form.totalSupply) <= 0) e.totalSupply = "Must be > 0";
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const next = () => {
    if (step === 1 && !validateStep1()) return;
    setStep((s) => s + 1);
  };

  const Field = ({ label, id, value, onChange, readOnly, error }: {
    label: string; id: keyof FormState; value: string;
    onChange?: React.ChangeEventHandler<HTMLInputElement>; readOnly?: boolean; error?: string;
  }) => (
    <div className="flex flex-col gap-1">
      <label className="text-sm text-gray-400">{label}</label>
      <input
        className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-indigo-500 disabled:opacity-50"
        value={value} onChange={onChange} readOnly={readOnly} disabled={readOnly}
      />
      {error && <span className="text-xs text-red-400">{error}</span>}
    </div>
  );

  return (
    <div className="bg-gray-900 border border-gray-700 rounded-2xl p-6 max-w-md mx-auto mt-6">
      {/* Progress */}
      <div className="flex items-center gap-2 mb-6">
        {STEPS.map((label, i) => (
          <div key={label} className="flex items-center gap-2">
            <div className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold
              ${step === i + 1 ? "bg-indigo-600 text-white" : step > i + 1 ? "bg-indigo-900 text-indigo-300" : "bg-gray-800 text-gray-500"}`}>
              {i + 1}
            </div>
            <span className={`text-xs hidden sm:block ${step === i + 1 ? "text-white" : "text-gray-500"}`}>{label}</span>
            {i < STEPS.length - 1 && <div className="w-4 h-px bg-gray-700 mx-1" />}
          </div>
        ))}
        <button onClick={onClose} className="ml-auto text-gray-500 hover:text-gray-300 text-lg leading-none">✕</button>
      </div>

      {/* Step 1 */}
      {step === 1 && (
        <div className="flex flex-col gap-4">
          <h3 className="text-white font-semibold">Token Details</h3>
          <Field label="Token Name" id="name" value={form.name} onChange={set("name")} error={errors.name} />
          <Field label="Symbol" id="symbol" value={form.symbol} onChange={set("symbol")} error={errors.symbol} />
          <Field label="Total Supply" id="totalSupply" value={form.totalSupply} onChange={set("totalSupply")} error={errors.totalSupply} />
        </div>
      )}

      {/* Step 2 */}
      {step === 2 && (
        <div className="flex flex-col gap-4">
          <h3 className="text-white font-semibold">Curve Parameters</h3>
          <Field label="Migration Target (XLM)" id="targetXlm" value={form.targetXlm} onChange={set("targetXlm")} />
          <Field label="Initial Virtual XLM" id="virtualXlm" value={form.virtualXlm} onChange={set("virtualXlm")} />
          <Field label="Creation Fee" id="name" value="10 XLM" readOnly />
        </div>
      )}

      {/* Step 3 */}
      {step === 3 && (
        <div className="flex flex-col gap-4">
          <h3 className="text-white font-semibold">Review & Deploy</h3>
          <div className="bg-gray-800 rounded-xl p-4 text-sm flex flex-col gap-2">
            {[
              ["Name", form.name], ["Symbol", form.symbol],
              ["Total Supply", form.totalSupply], ["Target XLM", form.targetXlm],
              ["Initial Virtual XLM", form.virtualXlm], ["Creation Fee", "10 XLM"],
            ].map(([k, v]) => (
              <div key={k} className="flex justify-between">
                <span className="text-gray-400">{k}</span>
                <span className="text-white font-medium">{v}</span>
              </div>
            ))}
          </div>
          <button
            onClick={() => alert("Connect wallet to deploy")}
            className="py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-semibold transition-colors"
          >
            🚀 Deploy
          </button>
        </div>
      )}

      {/* Navigation */}
      <div className="flex justify-between mt-6">
        {step > 1
          ? <button onClick={() => setStep((s) => s - 1)} className="px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-sm">Back</button>
          : <div />}
        {step < 3 && (
          <button onClick={next} className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-sm font-medium">Next →</button>
        )}
      </div>
    </div>
  );
}
