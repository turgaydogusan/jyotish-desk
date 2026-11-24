import { vi } from "vitest";

vi.mock("./bindings", () => ({
  commands: {
    engineStatus: vi.fn(),
    planetLongitudes: vi.fn(),
    printWindow: vi.fn(),
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
