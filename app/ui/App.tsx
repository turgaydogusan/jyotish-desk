import { useEffect, useState } from "react";
import { commands, type CommandError, type PlanetRow } from "./bindings";

/** Unix milisaniyesini UT Jülyen gününe çevirir (1970-01-01T00:00Z = JD 2440587.5). */
export function unixMsToJdUt(ms: number): number {
  return ms / 86_400_000 + 2440587.5;
}

export function App() {
  const [version, setVersion] = useState<string | null>(null);
  const [rows, setRows] = useState<PlanetRow[]>([]);
  const [error, setError] = useState<CommandError | null>(null);

  useEffect(() => {
    void (async () => {
      const status = await commands.engineStatus();
      if (status.status === "ok") setVersion(status.data.seVersion);
      else setError(status.error);
      const planets = await commands.planetLongitudes(unixMsToJdUt(Date.now()));
      if (planets.status === "ok") setRows(planets.data);
      else setError(planets.error);
    })();
  }, []);

  return (
    <main>
      <h1>Jyotish Desk</h1>
      <p>Swiss Ephemeris {version ?? "…"}</p>
      {error && <p role="alert">{error.code}</p>}
      <table>
        <thead>
          <tr>
            <th>Graha</th>
            <th>Sidereal lon (Lahiri)</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.body}>
              <td>{r.body}</td>
              <td>{r.lon.toFixed(4)}</td>
              <td>{r.retrograde ? "R" : ""}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </main>
  );
}
