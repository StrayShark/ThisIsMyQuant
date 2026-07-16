import { useState } from "react";
import { Database, ChevronDown, ChevronUp } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DataQualityBadge } from "@/components/layout/DataQualityBadge";
import type { DataDomain } from "@/types";
import { DataDomainActions } from "./DataDomainActions";
import { DOMAIN_LABELS, formatBytes, formatDateTime } from "./utils";

interface DataDomainGridProps {
  domains: DataDomain[];
}

export function DataDomainGrid({ domains }: DataDomainGridProps) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const toggle = (code: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(code)) next.delete(code);
      else next.add(code);
      return next;
    });
  };

  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {domains.map((domain) => {
        const isExpanded = expanded.has(domain.code);
        return (
          <Card
            key={domain.code}
            className="cursor-pointer transition-shadow hover:shadow-md"
            onClick={() => toggle(domain.code)}
          >
            <CardHeader className="pb-2">
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-2">
                  <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10 text-primary">
                    <Database className="size-4" aria-hidden="true" />
                  </div>
                  <div>
                    <CardTitle className="text-sm font-semibold">
                      {DOMAIN_LABELS[domain.code] ?? domain.name}
                    </CardTitle>
                    <p className="text-xs text-muted-foreground">{domain.source}</p>
                  </div>
                </div>
                {isExpanded ? (
                  <ChevronUp className="size-4 text-muted-foreground" aria-hidden="true" />
                ) : (
                  <ChevronDown className="size-4 text-muted-foreground" aria-hidden="true" />
                )}
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <p className="text-xs text-muted-foreground">记录数</p>
                  <p className="font-medium tabular-nums">{domain.record_count.toLocaleString()}</p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">大小</p>
                  <p className="font-medium tabular-nums">{formatBytes(domain.size_bytes)}</p>
                </div>
              </div>
              <div className="flex items-center justify-between">
                <DataQualityBadge status={domain.quality} />
                <span className="text-xs text-muted-foreground">
                  {formatDateTime(domain.last_updated)}
                </span>
              </div>
              {isExpanded && (
                <div className="space-y-3 border-t pt-3" onClick={(e) => e.stopPropagation()}>
                  <p className="text-xs text-muted-foreground">{domain.description}</p>
                  {domain.time_range && (
                    <div className="text-xs">
                      <span className="text-muted-foreground">时间范围：</span>
                      <span className="tabular-nums">
                        {domain.time_range.start ? formatDateTime(domain.time_range.start) : "—"} ~{" "}
                        {domain.time_range.end ? formatDateTime(domain.time_range.end) : "—"}
                      </span>
                    </div>
                  )}
                  <DataDomainActions domain={domain.code} size="sm" />
                </div>
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
