import { screen } from "@testing-library/react";
import { renderWithStores } from "../test/render";
import { PrintView } from "./PrintView";

const retroCell = (body: string) => screen.getByRole("cell", { name: body }).closest("tr")!.querySelectorAll("td")[2];

test("print table marks retrograde grahas with R but not the always-retrograde nodes", () => {
  renderWithStores(<PrintView />);
  expect(retroCell("Saturn")).toHaveTextContent(/^R$/);
  expect(retroCell("Rahu")).toHaveTextContent(/^$/);
  expect(retroCell("Ketu")).toHaveTextContent(/^$/);
});
