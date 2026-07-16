import { cn } from "@/lib/utils";

interface MiniSparklineProps {
  data?: number[] | null;
  width?: number;
  height?: number;
  className?: string;
}

export function MiniSparkline({
  data,
  width = 96,
  height = 32,
  className,
}: MiniSparklineProps) {
  const points = data ?? [];
  if (points.length < 2) {
    return <div className={cn("bg-muted/30", className)} style={{ width, height }} />;
  }

  const min = Math.min(...points);
  const max = Math.max(...points);
  const range = max - min || 1;

  const pathPoints = points.map((value, index) => {
    const x = (index / (points.length - 1)) * width;
    const y = height - ((value - min) / range) * (height - 4) - 2;
    return [x, y] as const;
  });

  const path = pathPoints
    .map(([x, y], i) => `${i === 0 ? "M" : "L"} ${x.toFixed(1)} ${y.toFixed(1)}`)
    .join(" ");

  const isUp = points[points.length - 1] >= points[0];
  const color = isUp ? "var(--color-up)" : "var(--color-down)";

  return (
    <svg
      className={cn("shrink-0", className)}
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      fill="none"
    >
      <path d={path} stroke={color} strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}
