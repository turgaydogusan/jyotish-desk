import { render, screen } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./bindings", () => ({
  commands: {
    engineStatus: vi.fn(async () => ({ status: "ok", data: { seVersion: "2.10.03" } })),
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [
        { body: "Sun", lon: 256.5, retrograde: false },
        { body: "Saturn", lon: 10.25, retrograde: true },
      ],
    })),
  },
}));

import { App, unixMsToJdUt } from "./App";

test("unixMsToJdUt maps the Unix epoch to JD 2440587.5", () => {
  expect(unixMsToJdUt(0)).toBe(2440587.5);
  expect(unixMsToJdUt(86_400_000)).toBe(2440588.5);
});

test("shows engine version and planet rows", async () => {
  render(<App />);
  expect(await screen.findByText(/2\.10\.03/)).toBeInTheDocument();
  expect(await screen.findByText("Sun")).toBeInTheDocument();
  expect(screen.getByText("256.5000")).toBeInTheDocument();
  expect(screen.getByText("R")).toBeInTheDocument();
});

test("shows error code when a command fails", async () => {
  const { commands } = await import("./bindings");
  vi.mocked(commands.planetLongitudes).mockResolvedValueOnce({
    status: "error",
    error: { code: "calc", params: { detail: "x" } },
  });
  render(<App />);
  expect(await screen.findByRole("alert")).toHaveTextContent("calc");
});
