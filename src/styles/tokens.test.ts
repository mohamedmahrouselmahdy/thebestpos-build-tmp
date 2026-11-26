import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(path.resolve(import.meta.dirname, "tokens.css"), "utf8");

const LIGHT: Record<string, string> = {
  "--bg": "#F3F6FC",
  "--surface": "#FFFFFF",
  "--surface-2": "#EAF0FA",
  "--line": "#D9E1F0",
  "--ink": "#0F1B33",
  "--ink-2": "#4B5A78",
  "--brand": "#1E40AF",
  "--brand-ink": "#FFFFFF",
  "--brand-soft": "#DFE8FF",
  "--ok": "#15803D",
  "--ok-soft": "#DCFCE7",
  "--warn": "#A15C07",
  "--warn-soft": "#FDECCB",
  "--danger": "#B42318",
  "--danger-soft": "#FEE4E2",
  "--info": "#0E6F8A",
  "--info-soft": "#D6F0F7",
  "--focus": "#2563EB",
  "--chart-1": "#2563EB",
};

const DARK: Record<string, string> = {
  "--bg": "#16181E",
  "--surface": "#20232B",
  "--surface-2": "#2A2E38",
  "--line": "#333846",
  "--ink": "#F2F6FA",
  "--ink-2": "#AAB4C3",
  "--brand": "#0B86B8",
  "--brand-ink": "#FFFFFF",
  "--brand-soft": "#0F3B4D",
  "--ok": "#4ADE80",
  "--ok-soft": "#10301D",
  "--warn": "#F5B94A",
  "--warn-soft": "#3A2A0A",
  "--danger": "#F87171",
  "--danger-soft": "#3B1517",
  "--info": "#5FC6E2",
  "--info-soft": "#10303A",
  "--focus": "#4FD3F7",
  "--chart-1": "#14B4E4",
};

function block(selector: string): Record<string, string> {
  const start = css.indexOf(`${selector} {`);
  if (start < 0) throw new Error(`missing block ${selector}`);
  const body = css.slice(start, css.indexOf("}", start));
  return Object.fromEntries(
    [...body.matchAll(/(--[a-z0-9-]+):\s*([^;]+);/g)].map((m) => [m[1]!, m[2]!.trim().toUpperCase()]),
  );
}

describe("design tokens (spec §10.1)", () => {
  it("defines every light value", () => {
    expect(block(":root")).toEqual(LIGHT);
  });

  it("defines every dark value on data-theme=dark", () => {
    expect(block(':root[data-theme="dark"]')).toEqual(DARK);
  });

  it("follows the system dark mode unless the user forced light", () => {
    expect(css).toContain("@media (prefers-color-scheme: dark)");
    expect(block(':root:not([data-theme="light"])')).toEqual(DARK);
  });
});
