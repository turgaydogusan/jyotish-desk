import { render, screen } from "@testing-library/react";
import { PrintSpike } from "./PrintSpike";

const rows = [
  { body: "Sun", sign: 8, lon: 256.5, showRetro: false },
  { body: "Moon", sign: 0, lon: 10.0, showRetro: false },
];

test("renders two A4 pages with chart svg and planet table", () => {
  const { container } = render(<PrintSpike rows={rows} />);
  expect(container.querySelectorAll("section.print-page")).toHaveLength(2);
  expect(container.querySelector("svg[data-testid='spike-chart']")).not.toBeNull();
  expect(screen.getByRole("cell", { name: "Sun" })).toBeInTheDocument();
  // Türkçe ve Devanagari karakterleri PDF'te gömülü fontla görünmeli.
  expect(screen.getByText(/Doğum haritası — ğüşıöç İ/)).toBeInTheDocument();
  expect(screen.getByText("राशि")).toBeInTheDocument();
});

test("planets are placed by the sign the backend sent", () => {
  render(<PrintSpike rows={rows} />);
  // Sun: Dhanu (index 8); hücrede "Sun" etiketi.
  const cell = screen.getByTestId("sign-8");
  expect(cell).toHaveTextContent("Sun");
});
