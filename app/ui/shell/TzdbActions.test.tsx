import { fireEvent, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { afterEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    updateChart: vi.fn(),
    keepStoredMoment: vi.fn(),
  },
}));

import { api } from "../api";
import type { OpenedClient } from "../bindings";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { NoticeBar, storeWarningNotices, useNotices } from "./notices";

afterEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

function tzdbOpened(): OpenedClient {
  const o = fixture().opened;
  o.warnings = [{ TzdbChanged: { stored_version: "2024a", current_version: "2026a", current_utc: "2000-01-01T12:00:00Z" } }];
  return o;
}

function Seed({ opened }: { opened: OpenedClient }) {
  const { push } = useNotices();
  useEffect(() => {
    for (const n of storeWarningNotices(opened.warnings)) push(n);
  }, [opened, push]);
  return <NoticeBar />;
}

test("accept re-saves the same birth data and reopens the chart without the band", async () => {
  const o = tzdbOpened();
  vi.mocked(api.updateChart).mockResolvedValue({ status: "ok", data: fixture().opened });
  renderWithStores(<Seed opened={o} />, { opened: o });
  expect(screen.getByRole("status")).toHaveTextContent("2024a → 2026a");
  fireEvent.click(screen.getByRole("button", { name: "Yeni zamanı kabul et" }));
  await waitFor(() => expect(api.updateChart).toHaveBeenCalledTimes(1));
  const [chartId, chart] = vi.mocked(api.updateChart).mock.calls[0];
  expect(chartId).toBe(o.chart_record.id);
  expect(chart).toEqual({ label: o.chart_record.label, kind: o.chart_record.kind, birth: o.chart_record.birth });
  await waitFor(() => expect(screen.queryByRole("button", { name: "Yeni zamanı kabul et" })).toBeNull());
});

test("keep stamps the stored moment and closes the band", async () => {
  const o = tzdbOpened();
  vi.mocked(api.keepStoredMoment).mockResolvedValue({ status: "ok", data: fixture().opened });
  renderWithStores(<Seed opened={o} />, { opened: o });
  fireEvent.click(screen.getByRole("button", { name: "Saklanan zamanı koru" }));
  await waitFor(() => expect(api.keepStoredMoment).toHaveBeenCalledWith(o.chart_record.id));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Saklanan zamanı koru" })).toBeNull());
  expect(api.updateChart).not.toHaveBeenCalled();
});

test("a failed decision keeps the band and reports the error", async () => {
  const o = tzdbOpened();
  vi.mocked(api.keepStoredMoment).mockResolvedValue({ status: "error", error: { code: "store_sqlite", params: { detail: "busy" } } });
  renderWithStores(<Seed opened={o} />, { opened: o });
  fireEvent.click(screen.getByRole("button", { name: "Saklanan zamanı koru" }));
  await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Saat dilimi kararı kaydedilemedi"));
  expect(screen.getByRole("button", { name: "Saklanan zamanı koru" })).toBeInTheDocument();
});
