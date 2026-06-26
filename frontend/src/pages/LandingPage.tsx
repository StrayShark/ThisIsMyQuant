import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { Sparkles } from "lucide-react";
import { api } from "@/api/client";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { NativeSelect } from "@/components/ui/native-select";
import { Badge } from "@/components/ui/badge";
import type { LlmCredentialInput, LlmProviderSetup } from "@/types";

type DraftEntry = {
  api_key: string;
  base_url: string;
  model: string;
  expanded: boolean;
};

function providerDraft(p: LlmProviderSetup): DraftEntry {
  return {
    api_key: "",
    base_url: p.base_url || p.default_base_url,
    model: p.model || p.default_model,
    expanded: p.name === "doubao" || p.configured,
  };
}

export function LandingPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [defaultProvider, setDefaultProvider] = useState("doubao");
  const [drafts, setDrafts] = useState<Record<string, DraftEntry>>({});
  const [error, setError] = useState("");

  const { data: setup, isLoading } = useQuery({
    queryKey: ["llm-setup"],
    queryFn: () => api.getLlmSetup(),
  });

  useEffect(() => {
    if (!setup) return;
    setDefaultProvider(setup.default_provider || "doubao");
    const next: Record<string, DraftEntry> = {};
    for (const p of setup.providers) {
      next[p.name] = providerDraft(p);
    }
    setDrafts(next);
  }, [setup]);

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (!setup) throw new Error("配置未加载");
      const credentials: LlmCredentialInput[] = [];
      for (const p of setup.providers) {
        const d = drafts[p.name];
        if (!d) continue;
        const key = d.api_key.trim();
        const isOllama = p.name === "ollama";
        if (!key && !isOllama && !p.configured) continue;
        if (!key && !isOllama) continue;
        credentials.push({
          provider: p.name,
          api_key: key,
          base_url: d.base_url.trim() || undefined,
          model: d.model.trim() || undefined,
        });
      }
      if (credentials.length === 0 && setup.setup_required) {
        throw new Error("请至少填写一个 LLM API Key，或配置 Ollama 地址");
      }
      return api.saveLlmSetup({ credentials, default_provider: defaultProvider });
    },
    onSuccess: (result) => {
      queryClient.setQueryData(["llm-setup"], result);
      queryClient.invalidateQueries({ queryKey: ["app-settings"] });
      queryClient.invalidateQueries({ queryKey: ["app-settings-bootstrap"] });
      if (!result.setup_required) {
        navigate("/", { replace: true });
      }
    },
    onError: (e) => {
      setError(e instanceof Error ? e.message : "保存失败");
    },
  });


  if (isLoading || !setup) {
    return (
      <div className="flex h-full items-center justify-center bg-background">
        <p className="text-sm text-muted-foreground">加载配置…</p>
      </div>
    );
  }

  return (
    <div className="page-scroll bg-canvas-soft">
      <div className="page-inner max-w-2xl space-y-6 py-12">
        <div className="space-y-2 text-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-xl border border-border bg-card">
            <Sparkles className="h-6 w-6 text-primary" />
          </div>
          <h1 className="text-2xl font-semibold tracking-tight text-foreground">
            配置大模型
          </h1>
          <p className="text-sm text-muted-foreground">
            API Key 加密保存在本地数据库，无需写入 .env。至少配置一个提供商即可开始分析。
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>默认分析模型</CardTitle>
          </CardHeader>
          <CardContent>
            <NativeSelect
              value={defaultProvider}
              onChange={(e) => setDefaultProvider(e.target.value)}
              className="w-full max-w-xs"
            >
              {setup.providers.map((p) => (
                <option key={p.name} value={p.name}>
                  {p.label}
                </option>
              ))}
            </NativeSelect>
          </CardContent>
        </Card>

        <div className="space-y-3">
          {setup.providers.map((p) => {
            const d = drafts[p.name] ?? providerDraft(p);
            return (
              <Card key={p.name}>
                <CardHeader className="flex-row items-center justify-between space-y-0">
                  <div className="flex items-center gap-2">
                    <CardTitle>{p.label}</CardTitle>
                    {p.configured && (
                      <Badge variant="up" className="text-[10px]">
                        已配置 {p.api_key_masked}
                      </Badge>
                    )}
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() =>
                      setDrafts((prev) => ({
                        ...prev,
                        [p.name]: { ...d, expanded: !d.expanded },
                      }))
                    }
                  >
                    {d.expanded ? "收起" : "配置"}
                  </Button>
                </CardHeader>
                {d.expanded && (
                  <CardContent className="space-y-3">
                    {p.key_required ? (
                      <div className="space-y-1">
                        <label className="text-xs text-muted-foreground">API Key</label>
                        <Input
                          type="password"
                          placeholder={p.configured ? "留空则保留现有 Key" : "粘贴 API Key"}
                          value={d.api_key}
                          onChange={(e) =>
                            setDrafts((prev) => ({
                              ...prev,
                              [p.name]: { ...d, api_key: e.target.value },
                            }))
                          }
                          className="font-mono"
                        />
                      </div>
                    ) : (
                      <p className="text-xs text-muted-foreground">
                        本地 Ollama 无需 Key，填写服务地址即可。
                      </p>
                    )}
                    <div className="grid gap-3 sm:grid-cols-2">
                      <div className="space-y-1">
                        <label className="text-xs text-muted-foreground">Base URL</label>
                        <Input
                          value={d.base_url}
                          onChange={(e) =>
                            setDrafts((prev) => ({
                              ...prev,
                              [p.name]: { ...d, base_url: e.target.value },
                            }))
                          }
                          className="font-mono text-xs"
                        />
                      </div>
                      <div className="space-y-1">
                        <label className="text-xs text-muted-foreground">Model</label>
                        <Input
                          value={d.model}
                          onChange={(e) =>
                            setDrafts((prev) => ({
                              ...prev,
                              [p.name]: { ...d, model: e.target.value },
                            }))
                          }
                          className="font-mono text-xs"
                        />
                      </div>
                    </div>
                  </CardContent>
                )}
              </Card>
            );
          })}
        </div>

        {error && <p className="text-sm text-down">{error}</p>}

        <div className="flex flex-wrap gap-3">
          <Button
            disabled={saveMutation.isPending}
            onClick={() => {
              setError("");
              saveMutation.mutate();
            }}
          >
            {saveMutation.isPending
              ? "保存中…"
              : setup.setup_required
                ? "保存并进入应用"
                : "保存配置"}
          </Button>
          {!setup.setup_required && (
            <Button variant="outline" onClick={() => navigate("/", { replace: true })}>
              进入应用
            </Button>
          )}
        </div>

        <p className="text-center text-xs text-muted-foreground">
          之后可在设置页的「大模型」区块重新配置 Key
        </p>
      </div>
    </div>
  );
}
