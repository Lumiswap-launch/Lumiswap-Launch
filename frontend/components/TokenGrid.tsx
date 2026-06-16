import TokenCard from "./TokenCard";

const LAUNCHES = [
  { name: "LumiToken",  creator: "GABC1234XYZ5678STELLAR001", xlm_raised: 42000,  target_xlm: 100000, price: 0.012 },
  { name: "NovaCoin",   creator: "GBDE9876QRS4321STELLAR002", xlm_raised: 75000,  target_xlm: 100000, price: 0.034 },
  { name: "StarDust",   creator: "GCFG5555MNO1122STELLAR003", xlm_raised: 10000,  target_xlm: 50000,  price: 0.007 },
  { name: "XLMRocket",  creator: "GDHK2222PQR9988STELLAR004", xlm_raised: 98000,  target_xlm: 100000, price: 0.091 },
];

export default function TokenGrid() {
  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
      {LAUNCHES.map((t) => <TokenCard key={t.name} {...t} />)}
    </div>
  );
}
