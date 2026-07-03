import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { BootstrapLoader } from "@/components/BootstrapLoader";
import { waitForAppReady } from "@/lib/tauri-bridge";
import { isTauriRuntime } from "@/lib/platform";
import { applyShellLayoutVars } from "@/lib/shell-layout";
import "./index.css";

const root = ReactDOM.createRoot(document.getElementById("root")!);

function applyPlatformClass() {
  const html = document.documentElement;
  if (isTauriRuntime()) {
    html.classList.add("tauri");
    if (/Mac|iPhone|iPod|iPad/.test(navigator.platform)) {
      html.classList.add("tauri-mac");
    }
  }
}

applyPlatformClass();
applyShellLayoutVars();

function render(node: React.ReactNode) {
  root.render(<React.StrictMode>{node}</React.StrictMode>);
}

async function bootstrap() {
  render(<BootstrapLoader />);
  await waitForAppReady();
  render(<App />);
}

bootstrap();
