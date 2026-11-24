import { render, screen } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./api", () => ({
  api: {
    engineStatus: vi.fn(async () => ({ status: "ok", data: { seVersion: "2.10.03" } })),
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [
        { body: "Sun", lon: 256.5, retrograde: false },
        { body: "Saturn", lon: 10.25, retrograde: true },
      ],
    })),
    printWindow: vi.fn(async () => ({ status: "ok", data: null })),
  },
}));

import { App } from "./App";

test("shows engine version and planet rows", async () => {
  render(<App />);
  expect(await screen.findByText(/2\.10\.03/)).toBeInTheDocument();
  expect(await screen.findByText("Sun")).toBeInTheDocument();
  expect(screen.getByText("256.5000")).toBeInTheDocument();
  expect(screen.getByText("R")).toBeInTheDocument();
});

test("shows error code when a command fails", async () => {
  const { api } = await import("./api");
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({
    status: "error",
    error: { code: "calc", params: { detail: "x" } },
  });
  render(<App />);
  expect(await screen.findByRole("alert")).toHaveTextContent("calc");
});

test("shows a dash when a longitude is not finite (serialized as null)", async () => {
  const { api } = await import("./api");
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({
    status: "ok",
    data: [{ body: "Mars", lon: null, retrograde: false }],
  });
  render(<App />);
  expect(await screen.findByText("Mars")).toBeInTheDocument();
  expect(screen.getByText("—")).toBeInTheDocument();
});
