#!/usr/bin/env node
/** 从 img/icon.jpg 生成 Tauri 图标与网页 favicon。 */
import sharp from "../frontend/node_modules/sharp/lib/index.js";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const src = path.join(root, "img/icon.jpg");
const outDir = path.join(root, "src-tauri/icons");
const pub = path.join(root, "frontend/public");

await mkdir(outDir, { recursive: true });
await mkdir(pub, { recursive: true });

const sizes = { "32x32.png": 32, "128x128.png": 128, "128x128@2x.png": 256, "icon.png": 512 };
for (const [name, sz] of Object.entries(sizes)) {
  await sharp(src).resize(sz, sz).ensureAlpha().png().toFile(path.join(outDir, name));
}
await sharp(src).resize(512, 512).ensureAlpha().png().toFile(path.join(pub, "icon.png"));
await sharp(src).resize(32, 32).ensureAlpha().png().toFile(path.join(pub, "favicon.png"));
await sharp(src).resize(180, 180).jpeg({ quality: 92 }).toFile(path.join(pub, "icon.jpg"));
console.log("icons generated");
