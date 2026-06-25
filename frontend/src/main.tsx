import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { BootstrapLoader } from "@/components/BootstrapLoader";
import { waitForAppReady } from "@/lib/tauri-bridge";
import "./index.css";

const root = ReactDOM.createRoot(document.getElementById("root")!);

function render(node: React.ReactNode) {
  root.render(<React.StrictMode>{node}</React.StrictMode>);
}

async function bootstrap() {
  render(<BootstrapLoader />);
  await waitForAppReady();
  render(<App />);
}

bootstrap();
