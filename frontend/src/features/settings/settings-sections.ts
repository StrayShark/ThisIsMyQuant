import type { LucideIcon } from "lucide-react";
import {
  Bot,
  CalendarClock,
  Database,
  FlaskConical,
  HardDrive,
  Palette,
  Scale,
  SlidersHorizontal,
  Wrench,
} from "lucide-react";

export type SettingsSectionId =
  | "appearance"
  | "schedule"
  | "preferences"
  | "data"
  | "llm"
  | "simulation"
  | "storage"
  | "experimental"
  | "debug";

export interface SettingsSection {
  id: SettingsSectionId;
  label: string;
  icon: LucideIcon;
}

export const SETTINGS_SECTIONS: SettingsSection[] = [
  { id: "appearance", label: "外观", icon: Palette },
  { id: "schedule", label: "定时任务", icon: CalendarClock },
  { id: "preferences", label: "运营配置", icon: SlidersHorizontal },
  { id: "data", label: "数据源", icon: Database },
  { id: "llm", label: "大模型", icon: Bot },
  { id: "simulation", label: "模拟规则", icon: Scale },
  { id: "storage", label: "存储与导出", icon: HardDrive },
  { id: "experimental", label: "实验区", icon: FlaskConical },
  { id: "debug", label: "调试", icon: Wrench },
];

export function defaultSettingsSection(): SettingsSectionId {
  return "schedule";
}

export function parseSettingsSection(raw: string | null): SettingsSectionId {
  const hit = SETTINGS_SECTIONS.find((s) => s.id === raw);
  return hit?.id ?? defaultSettingsSection();
}

export function settingsSectionMeta(id: SettingsSectionId): SettingsSection {
  return SETTINGS_SECTIONS.find((s) => s.id === id) ?? SETTINGS_SECTIONS[0];
}

const RETURN_KEY = "settings-return-path";

export function saveSettingsReturnPath(path: string) {
  sessionStorage.setItem(RETURN_KEY, path || "/");
}

export function consumeSettingsReturnPath(): string {
  const saved = sessionStorage.getItem(RETURN_KEY);
  sessionStorage.removeItem(RETURN_KEY);
  return saved && saved !== "/settings" ? saved : "/";
}
