import { act, render, screen } from "@testing-library/react";
import { renderToStaticMarkup } from "react-dom/server";
import { NoticeBar, NoticeProvider, type Notice } from "./notices";

const boot: Notice = { id: "boot", type: "error", messageKey: "notice.openFailed", error: { code: "store_sqlite", params: { detail: "disk" } } };

test("a notice present at start is added after the first paint, so it is announced", async () => {
  const tree = (
    <NoticeProvider initial={[boot]}>
      <NoticeBar />
    </NoticeProvider>
  );
  // İlk boyama (sunucu çıktısı ilk render'a eşittir): bant var, bildirim yok.
  const first = renderToStaticMarkup(tree);
  expect(first).not.toContain('role="alert"');
  render(tree);
  await act(async () => {});
  expect(screen.getByRole("alert")).toHaveTextContent("disk");
});
