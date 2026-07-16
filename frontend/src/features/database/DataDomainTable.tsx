import { DataQualityBadge } from "@/components/layout/DataQualityBadge";
import type { DataDomain } from "@/types";
import { DataDomainActions } from "./DataDomainActions";
import { DOMAIN_LABELS, formatBytes, formatDateTime } from "./utils";

interface DataDomainTableProps {
  domains: DataDomain[];
}

export function DataDomainTable({ domains }: DataDomainTableProps) {
  return (
    <div className="rounded-2xl border">
      <table className="w-full text-sm">
        <thead className="bg-muted/40">
          <tr>
            <th className="px-4 py-3 text-left font-medium">域</th>
            <th className="px-4 py-3 text-right font-medium">记录数</th>
            <th className="px-4 py-3 text-left font-medium">时间范围</th>
            <th className="px-4 py-3 text-left font-medium">来源</th>
            <th className="px-4 py-3 text-left font-medium">质量</th>
            <th className="px-4 py-3 text-right font-medium">大小</th>
            <th className="px-4 py-3 text-left font-medium">操作</th>
          </tr>
        </thead>
        <tbody>
          {domains.map((domain) => (
            <tr key={domain.code} className="border-t">
              <td className="px-4 py-3">
                <div className="font-medium">{DOMAIN_LABELS[domain.code] ?? domain.name}</div>
                <div className="text-xs text-muted-foreground">{domain.code}</div>
              </td>
              <td className="px-4 py-3 text-right tabular-nums">
                {domain.record_count.toLocaleString()}
              </td>
              <td className="px-4 py-3 text-xs tabular-nums text-muted-foreground">
                {domain.time_range ? (
                  <>
                    <div>{domain.time_range.start ? formatDateTime(domain.time_range.start) : "—"}</div>
                    <div>{domain.time_range.end ? formatDateTime(domain.time_range.end) : "—"}</div>
                  </>
                ) : (
                  "—"
                )}
              </td>
              <td className="px-4 py-3 text-muted-foreground">{domain.source}</td>
              <td className="px-4 py-3">
                <DataQualityBadge status={domain.quality} />
              </td>
              <td className="px-4 py-3 text-right tabular-nums">{formatBytes(domain.size_bytes)}</td>
              <td className="px-4 py-3">
                <DataDomainActions domain={domain.code} size="sm" />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
