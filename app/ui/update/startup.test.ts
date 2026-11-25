import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { checkForUpdate: vi.fn() } }));

import { api } from "../api";
import { saveAutoCheck } from "./pref";
import { startupUpdateCheck } from "./startup";

afterEach(() => vi.mocked(api.checkForUpdate).mockReset());

test("without consent nothing is asked of the network", async () => {
  const push = vi.fn();
  await startupUpdateCheck(push);
  saveAutoCheck(false);
  await startupUpdateCheck(push);
  expect(api.checkForUpdate).not.toHaveBeenCalled();
  expect(push).not.toHaveBeenCalled();
});

test("with consent a newer version becomes one info notice", async () => {
  saveAutoCheck(true);
  vi.mocked(api.checkForUpdate).mockResolvedValue({ status: "ok", data: { version: "0.9.1", notes: null, published: null } });
  const push = vi.fn();
  await startupUpdateCheck(push);
  expect(api.checkForUpdate).toHaveBeenCalledTimes(1);
  expect(push).toHaveBeenCalledWith({ id: "update", type: "info", messageKey: "notice.updateAvailable", params: { version: "0.9.1" } });
});

test("an up-to-date app and a failed check are both silent", async () => {
  saveAutoCheck(true);
  const push = vi.fn();
  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "ok", data: null });
  await startupUpdateCheck(push);
  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "error", error: { code: "update_check_failed", params: { reason: "network" } } });
  await startupUpdateCheck(push);
  expect(push).not.toHaveBeenCalled();
});
