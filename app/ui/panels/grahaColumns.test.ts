import { DEFAULT_GRAHA_COLUMNS, grahaColumns, toggleColumn } from "./grahaColumns";

test("missing, empty or garbage settings fall back to the default columns", () => {
  expect(grahaColumns({})).toEqual(DEFAULT_GRAHA_COLUMNS);
  expect(grahaColumns({ columns: [] })).toEqual(DEFAULT_GRAHA_COLUMNS);
  expect(grahaColumns({ columns: "sign" })).toEqual(DEFAULT_GRAHA_COLUMNS);
  expect(grahaColumns({ columns: ["bogus", 3] })).toEqual(DEFAULT_GRAHA_COLUMNS);
});

test("known columns are kept in canonical order without duplicates", () => {
  expect(grahaColumns({ columns: ["house", "sign", "x", "sign", "lon"] })).toEqual(["sign", "lon", "house"]);
});

test("toggling adds in canonical order, removes, and never empties the table", () => {
  expect(toggleColumn(["sign", "house"], "lon")).toEqual(["sign", "lon", "house"]);
  expect(toggleColumn(["sign", "house"], "sign")).toEqual(["house"]);
  expect(toggleColumn(["house"], "house")).toEqual(["house"]);
});
