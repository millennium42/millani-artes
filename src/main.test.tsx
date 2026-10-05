import { type ReactNode, StrictMode } from "react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

const { createRoot, render } = vi.hoisted(() => {
  const render = vi.fn<(children: ReactNode) => void>();
  return { createRoot: vi.fn(() => ({ render })), render };
});

vi.mock("react-dom/client", () => ({ createRoot }));

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

test("falha com mensagem legível quando o root está ausente", async () => {
  const getElementById = vi.fn().mockReturnValue(null);
  vi.stubGlobal("document", { getElementById });

  await expect(import("./main")).rejects.toThrow(
    "Não foi possível iniciar Millani Artes.",
  );
  expect(getElementById).toHaveBeenCalledExactlyOnceWith("root");
  expect(createRoot).not.toHaveBeenCalled();
  expect(render).not.toHaveBeenCalled();
});

test("monta App em StrictMode no root encontrado", async () => {
  const root = { id: "root" };
  const getElementById = vi.fn().mockReturnValue(root);
  vi.stubGlobal("document", { getElementById });

  await import("./main");
  const { default: App } = await import("./App");

  expect(getElementById).toHaveBeenCalledExactlyOnceWith("root");
  expect(createRoot).toHaveBeenCalledExactlyOnceWith(root);
  expect(render).toHaveBeenCalledExactlyOnceWith(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});
