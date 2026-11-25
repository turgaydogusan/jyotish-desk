import { vi } from "vitest";

vi.mock("./bindings", () => ({
  commands: {
    engineStatus: vi.fn(),
    planetLongitudes: vi.fn(),
    printWindow: vi.fn(),
    restoreSession: vi.fn(),
    createClient: vi.fn(),
    updateChart: vi.fn(),
    saveViewState: vi.fn(),
    loadPanelLayout: vi.fn(),
    savePanelLayout: vi.fn(),
    dashaChildren: vi.fn(),
    searchPlaces: vi.fn(),
    previewMoment: vi.fn(),
  },
}));

import { api } from "./api";
import { commands } from "./bindings";

test("passes ok results through", async () => {
  vi.mocked(commands.engineStatus).mockResolvedValueOnce({
    status: "ok",
    data: { seVersion: "2.10.03" },
  });
  expect(await api.engineStatus()).toEqual({ status: "ok", data: { seVersion: "2.10.03" } });
});

test("passes real CommandError objects through", async () => {
  const error = { code: "calc", params: { detail: "x" } };
  vi.mocked(commands.planetLongitudes).mockResolvedValueOnce({ status: "error", error });
  expect(await api.planetLongitudes(1)).toEqual({ status: "error", error });
});

test("normalizes a string rejection into an ipc error", async () => {
  vi.mocked(commands.printWindow).mockResolvedValueOnce({
    status: "error",
    error: "command print_window not found" as never,
  });
  expect(await api.printWindow()).toEqual({
    status: "error",
    error: { code: "ipc", params: { detail: "command print_window not found" } },
  });
});

test("catches thrown errors into an ipc error", async () => {
  vi.mocked(commands.engineStatus).mockRejectedValueOnce(new Error("boom"));
  expect(await api.engineStatus()).toEqual({
    status: "error",
    error: { code: "ipc", params: { detail: "Error: boom" } },
  });
});

test("every wrapper forwards its arguments to the command of the same name", async () => {
  const calls: [keyof typeof api, unknown[]][] = [
    ["restoreSession", []],
    ["createClient", [{ name: "A" }]],
    ["updateChart", [3, { label: "" }]],
    ["saveViewState", [3, { schema_version: 1 }]],
    ["loadPanelLayout", []],
    ["savePanelLayout", [null]],
    ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
    ["searchPlaces", ["delhi", 8]],
    ["previewMoment", [{ date: "2000-01-01" }]],
  ];
  for (const [name, args] of calls) {
    const command = vi.mocked(commands[name] as (...a: unknown[]) => Promise<unknown>);
    command.mockResolvedValueOnce({ status: "ok", data: name });
    const call = api[name] as (...a: unknown[]) => Promise<unknown>;
    expect(await call(...args)).toEqual({ status: "ok", data: name });
    expect(command).toHaveBeenCalledWith(...args);
  }
});

test("import and export commands are not reachable from the UI (Plan 08 deferred items 1 and 3)", () => {
  for (const name of ["importJson", "importJhd", "exportJson", "exportCsv"]) {
    expect(Object.keys(api)).not.toContain(name);
  }
});
