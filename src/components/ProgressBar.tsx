type Props = {
  className?: string;
};

export default function ProgressBar({ className = "" }: Props) {
  return (
    <div className={["w-full", className].join(" ")}> 
      <div className="progress">
        <div className="bar" />
      </div>
    </div>
  );
}

