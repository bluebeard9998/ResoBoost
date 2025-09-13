type Props = {
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
  "aria-label"?: string;
};

export default function Stepper({ value, onChange, min = 1, max, step = 1, ...rest }: Props) {
  const dec = () => onChange(Math.max(min, value - step));
  const inc = () => onChange(Math.min(max ?? Number.MAX_SAFE_INTEGER, value + step));

  return (
    <div className="stepper" {...rest}>
      <button type="button" onClick={dec} className="px-3 text-xl text-gray-300 hover:text-white" aria-label="decrease">-</button>
      <div className="w-10 text-center font-semibold select-none">{value}</div>
      <button type="button" onClick={inc} className="px-3 text-xl text-gray-300 hover:text-white" aria-label="increase">+</button>
    </div>
  );
}


