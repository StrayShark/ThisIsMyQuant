import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import { api } from "@/api/client";
import type { SimContractRule, SimRiskRule } from "@/types";

const emptyRule = (): SimContractRule => ({
  symbol: "",
  name: "",
  exchange: "SHFE",
  contract_multiplier: 10,
  price_tick: 1,
  margin_rate_long: 0.1,
  margin_rate_short: 0.1,
  commission_mode: "per_hand",
  commission_open: 0,
  commission_close: 0,
  commission_close_today: 0,
  min_order_qty: 1,
  lot_size: 1,
  max_order_qty: 0,
  daily_price_limit_up: 0,
  daily_price_limit_down: 0,
  default_slippage_ticks: 0,
  is_custom: true,
  updated_at: new Date().toISOString(),
});

const emptyRiskRule = (accountId: string): SimRiskRule => ({
  id: "",
  account_id: accountId,
  scope: "account",
  symbol: null,
  rule_type: "risk_ratio",
  threshold: 0.9,
  action: "block_open",
  enabled: true,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
});

export function SimulationRulesPanel() {
  const qc = useQueryClient();
  const [editing, setEditing] = useState<SimContractRule | null>(null);
  const [riskEditing, setRiskEditing] = useState<SimRiskRule | null>(null);

  const { data: rules = [], isLoading: rulesLoading } = useQuery({
    queryKey: ["sim-contract-rules"],
    queryFn: () => api.listSimContractRules(),
  });

  const { data: accounts = [] } = useQuery({
    queryKey: ["sim-accounts"],
    queryFn: () => api.listSimAccounts(),
  });

  const defaultAccount = accounts[0];

  const { data: riskRules = [] } = useQuery({
    queryKey: ["sim-risk-rules", defaultAccount?.id],
    queryFn: () => api.listSimRiskRules({ account_id: defaultAccount?.id }),
    enabled: !!defaultAccount,
  });

  const saveRule = useMutation({
    mutationFn: api.saveSimContractRule,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sim-contract-rules"] });
      setEditing(null);
    },
  });

  const deleteRule = useMutation({
    mutationFn: api.deleteSimContractRule,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["sim-contract-rules"] }),
  });

  const saveRisk = useMutation({
    mutationFn: api.saveSimRiskRule,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sim-risk-rules"] });
      setRiskEditing(null);
    },
  });

  const deleteRisk = useMutation({
    mutationFn: api.deleteSimRiskRule,
    onSuccess: () => qc.invalidateQueries({ queryKey: ["sim-risk-rules"] }),
  });

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-semibold">合约规则</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {rulesLoading ? (
            <p className="text-sm text-muted-foreground">加载中…</p>
          ) : (
            <div className="space-y-2">
              {rules.map((r) => (
                <div
                  key={r.symbol}
                  className="flex items-center justify-between rounded-md border border-border px-3 py-2"
                >
                  <div className="text-sm">
                    <span className="font-medium">{r.symbol}</span>
                    <span className="ml-2 text-muted-foreground">{r.name}</span>
                    <span className="ml-2 font-mono text-xs text-muted-foreground">
                      ×{r.contract_multiplier} 保证金 {r.margin_rate_long * 100}%
                    </span>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setEditing({ ...r })}
                  >
                    编辑
                  </Button>
                </div>
              ))}
            </div>
          )}
          <Button variant="outline" size="sm" onClick={() => setEditing(emptyRule())}>
            + 新增规则
          </Button>
        </CardContent>
      </Card>

      {editing && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">
              {editing.symbol ? `编辑 ${editing.symbol}` : "新增合约规则"}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">合约代码</label>
                <Input
                  value={editing.symbol}
                  onChange={(e) => setEditing({ ...editing, symbol: e.target.value.toUpperCase() })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">名称</label>
                <Input
                  value={editing.name}
                  onChange={(e) => setEditing({ ...editing, name: e.target.value })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">合约乘数</label>
                <Input
                  type="number"
                  value={editing.contract_multiplier}
                  onChange={(e) => setEditing({ ...editing, contract_multiplier: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">最小变动价位</label>
                <Input
                  type="number"
                  step={0.01}
                  value={editing.price_tick}
                  onChange={(e) => setEditing({ ...editing, price_tick: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">保证金率(多)</label>
                <Input
                  type="number"
                  step={0.01}
                  value={editing.margin_rate_long}
                  onChange={(e) => setEditing({ ...editing, margin_rate_long: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">保证金率(空)</label>
                <Input
                  type="number"
                  step={0.01}
                  value={editing.margin_rate_short}
                  onChange={(e) => setEditing({ ...editing, margin_rate_short: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">开仓手续费</label>
                <Input
                  type="number"
                  step={0.1}
                  value={editing.commission_open}
                  onChange={(e) => setEditing({ ...editing, commission_open: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">平仓手续费</label>
                <Input
                  type="number"
                  step={0.1}
                  value={editing.commission_close}
                  onChange={(e) => setEditing({ ...editing, commission_close: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">平今手续费</label>
                <Input
                  type="number"
                  step={0.1}
                  value={editing.commission_close_today}
                  onChange={(e) => setEditing({ ...editing, commission_close_today: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">默认滑点(tick)</label>
                <Input
                  type="number"
                  step={0.1}
                  value={editing.default_slippage_ticks}
                  onChange={(e) => setEditing({ ...editing, default_slippage_ticks: Number(e.target.value) })}
                />
              </div>
            </div>
            <div className="flex gap-2 pt-2">
              <Button size="sm" onClick={() => saveRule.mutate(editing)} disabled={saveRule.isPending}>
                保存
              </Button>
              <Button variant="outline" size="sm" onClick={() => setEditing(null)}>
                取消
              </Button>
              {editing.symbol && (
                <Button
                  variant="destructive"
                  size="sm"
                  className="ml-auto"
                  onClick={() => deleteRule.mutate(editing.symbol)}
                  disabled={deleteRule.isPending}
                >
                  删除
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-semibold">风控规则</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {riskRules.length === 0 ? (
            <p className="text-sm text-muted-foreground">暂无风控规则</p>
          ) : (
            <div className="space-y-2">
              {riskRules.map((r) => (
                <div
                  key={r.id}
                  className="flex items-center justify-between rounded-md border border-border px-3 py-2"
                >
                  <div className="text-sm">
                    <span className="font-medium">{r.rule_type}</span>
                    <span className="ml-2 text-muted-foreground">
                      {r.scope === "symbol" && r.symbol ? `${r.symbol} ` : ""}
                      阈值 {r.threshold} · 动作 {r.action}
                    </span>
                    {!r.enabled && (
                      <span className="ml-2 text-xs text-muted-foreground">(已禁用)</span>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <Button variant="ghost" size="sm" onClick={() => setRiskEditing({ ...r })}>
                      编辑
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-destructive"
                      onClick={() => deleteRisk.mutate(r.id)}
                    >
                      删除
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
          {defaultAccount && (
            <Button variant="outline" size="sm" onClick={() => setRiskEditing(emptyRiskRule(defaultAccount.id))}>
              + 新增风控规则
            </Button>
          )}
        </CardContent>
      </Card>

      {riskEditing && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-semibold">
              {riskEditing.id ? "编辑风控规则" : "新增风控规则"}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">范围</label>
                <NativeSelect
                  value={riskEditing.scope}
                  onChange={(e) =>
                    setRiskEditing({ ...riskEditing, scope: e.target.value as SimRiskRule["scope"] })
                  }
                >
                  <option value="account">账户级</option>
                  <option value="symbol">品种级</option>
                </NativeSelect>
              </div>
              {riskEditing.scope === "symbol" && (
                <div className="space-y-1">
                  <label className="text-xs text-muted-foreground">品种</label>
                  <Input
                    value={riskEditing.symbol ?? ""}
                    onChange={(e) =>
                      setRiskEditing({ ...riskEditing, symbol: e.target.value || null })
                    }
                  />
                </div>
              )}
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">规则类型</label>
                <NativeSelect
                  value={riskEditing.rule_type}
                  onChange={(e) =>
                    setRiskEditing({ ...riskEditing, rule_type: e.target.value as SimRiskRule["rule_type"] })
                  }
                >
                  <option value="max_lots">最大手数</option>
                  <option value="symbol_margin_ratio">单品种保证金占比</option>
                  <option value="risk_ratio">账户风险度</option>
                  <option value="loss_limit">亏损限额</option>
                </NativeSelect>
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">阈值</label>
                <Input
                  type="number"
                  step={0.01}
                  value={riskEditing.threshold}
                  onChange={(e) => setRiskEditing({ ...riskEditing, threshold: Number(e.target.value) })}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">动作</label>
                <NativeSelect
                  value={riskEditing.action}
                  onChange={(e) =>
                    setRiskEditing({ ...riskEditing, action: e.target.value as SimRiskRule["action"] })
                  }
                >
                  <option value="reject">拒单</option>
                  <option value="block_open">禁止开仓</option>
                  <option value="force_liquidate">强制平仓</option>
                </NativeSelect>
              </div>
              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={riskEditing.enabled}
                  onChange={(e) => setRiskEditing({ ...riskEditing, enabled: e.target.checked })}
                />
                启用
              </label>
            </div>
            <div className="flex gap-2 pt-2">
              <Button size="sm" onClick={() => saveRisk.mutate(riskEditing)} disabled={saveRisk.isPending}>
                保存
              </Button>
              <Button variant="outline" size="sm" onClick={() => setRiskEditing(null)}>
                取消
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
