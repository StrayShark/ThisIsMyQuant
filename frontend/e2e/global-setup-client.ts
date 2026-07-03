import { spawn, type ChildProcess } from "node:child_process";
import { execSync } from "node:child_process";
import { existsSync, readFileSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
const READY_FILE = path.join(ROOT, "data", "e2e-ready.json");
const E2E_HTTP = "http://127.0.0.1:17845";
const START_TIMEOUT_MS = 180_000;

let tauriProc: ChildProcess | null = null;

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitForReady() {
  const deadline = Date.now() + START_TIMEOUT_MS;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`${E2E_HTTP}/health`);
      if (res.ok && existsSync(READY_FILE)) {
        const data = JSON.parse(readFileSync(READY_FILE, "utf8")) as {
          ready?: boolean;
          llm_providers?: string[];
        };
        if (data.ready && (data.llm_providers?.length ?? 0) > 0) {
          return data;
        }
      }
    } catch {
      /* not up yet */
    }
    await sleep(2000);
  }
  throw new Error(`Tauri 客户端未在 ${START_TIMEOUT_MS / 1000}s 内就绪，请检查 LLM Key 与 pnpm tauri:dev 日志`);
}

export default async function globalSetup() {
  console.log("[e2e-client] 同步 .env …");
  execSync("bash scripts/sync-env.sh", { cwd: ROOT, stdio: "inherit" });
  rmSync(READY_FILE, { force: true });

  console.log("[e2e-client] 启动 Tauri 客户端 …");
  tauriProc = spawn("pnpm", ["tauri:dev"], {
    cwd: ROOT,
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env },
    detached: true,
  });

  tauriProc.stdout?.on("data", (buf: Buffer) => {
    const line = buf.toString();
    if (line.includes("ThisIsMyQuant ready") || line.includes("e2e http")) {
      process.stdout.write(`[tauri] ${line}`);
    }
  });
  tauriProc.stderr?.on("data", (buf: Buffer) => {
    const line = buf.toString();
    if (line.toLowerCase().includes("error")) {
      process.stderr.write(`[tauri] ${line}`);
    }
  });

  if (tauriProc.pid) {
    process.env.TAURI_E2E_PID = String(tauriProc.pid);
  }

  const ready = await waitForReady();
  console.log("[e2e-client] 客户端就绪:", ready);
}

export async function stopClientProcess() {
  const pid = process.env.TAURI_E2E_PID;
  if (!pid) return;
  console.log("[e2e-client] 停止 Tauri 进程", pid);
  try {
    if (process.platform !== "win32") {
      process.kill(-Number(pid), "SIGTERM");
    } else {
      process.kill(Number(pid), "SIGTERM");
    }
  } catch {
    /* already stopped */
  }
  try {
    spawn("lsof", ["-ti:5173"], { stdio: ["ignore", "pipe", "ignore"] }).stdout?.on("data", (d) => {
      const pids = d.toString().trim().split("\n").filter(Boolean);
      for (const p of pids) {
        try {
          process.kill(Number(p), "SIGKILL");
        } catch {
          /* ignore */
        }
      }
    });
  } catch {
    /* ignore */
  }
}
