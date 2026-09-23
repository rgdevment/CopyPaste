import type { Kept, Look } from "../core";
import { Band, Knob, Line } from "./Bits";

export default function General({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => void;
}) {
  return (
    <>
      <h1>General</h1>

      <Band says="Ventana" />

      <Line says="Idioma" why="El de tu equipo, salvo que elijas otro">
        <select
          aria-label="Idioma"
          value={kept.locale ?? ""}
          onChange={(event) => change({ locale: event.target.value || null })}
        >
          <option value="">El del sistema</option>
          <option value="es">Español</option>
          <option value="en">English</option>
        </select>
      </Line>

      <Line says="Tema" why="Claro, oscuro, o el que use tu equipo">
        <select
          aria-label="Tema"
          value={kept.theme}
          onChange={(event) => change({ theme: event.target.value as Look })}
        >
          <option value="system">El del sistema</option>
          <option value="light">Claro</option>
          <option value="dark">Oscuro</option>
        </select>
      </Line>

      <Line
        says="Arranca con la sesión"
        why="CopyPaste se abre al iniciar tu equipo y espera en la bandeja"
      >
        <Knob
          on={kept["wakes-with-session"]}
          says="Arranca con la sesión"
          onPress={() => change({ "wakes-with-session": !kept["wakes-with-session"] })}
        />
      </Line>

      <Line
        says="Atajo del panel"
        why="Presiónalo en cualquier parte y el panel aparece donde estés escribiendo"
      >
        <span className="keys">{kept.shortcut.replaceAll("+", " + ")}</span>
        <button type="button" className="mild" disabled>
          Cambiar
        </button>
      </Line>

      <Line says="Ocultar al perder el foco" why="El panel se va solo en cuanto tocas otra ventana">
        <Knob
          on={kept["hides-when-left"]}
          says="Ocultar al perder el foco"
          onPress={() => change({ "hides-when-left": !kept["hides-when-left"] })}
        />
      </Line>
    </>
  );
}
