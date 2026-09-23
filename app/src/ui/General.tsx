import { useState } from "react";
import { Band, Knob, Line } from "./Bits";

export default function General() {
  const [wakes, setWakes] = useState(true);
  const [hides, setHides] = useState(true);
  const [locale, setLocale] = useState("");
  const [theme, setTheme] = useState("");

  return (
    <>
      <h1>General</h1>

      <Band says="Ventana" />

      <Line says="Idioma" why="El de tu equipo, salvo que elijas otro">
        <select
          aria-label="Idioma"
          value={locale}
          onChange={(event) => setLocale(event.target.value)}
        >
          <option value="">El del sistema</option>
          <option value="es">Español</option>
          <option value="en">English</option>
        </select>
      </Line>

      <Line says="Tema" why="Claro, oscuro, o el que use tu equipo">
        <select
          aria-label="Tema"
          value={theme}
          onChange={(event) => {
            setTheme(event.target.value);
            const root = document.documentElement;
            if (event.target.value) {
              root.setAttribute("data-theme", event.target.value);
            } else {
              root.removeAttribute("data-theme");
            }
          }}
        >
          <option value="">El del sistema</option>
          <option value="light">Claro</option>
          <option value="dark">Oscuro</option>
        </select>
      </Line>

      <Line
        says="Arranca con la sesión"
        why="CopyPaste se abre al iniciar tu equipo y espera en la bandeja"
      >
        <Knob on={wakes} says="Arranca con la sesión" onPress={() => setWakes(!wakes)} />
      </Line>

      <Line
        says="Atajo del panel"
        why="Presiónalo en cualquier parte y el panel aparece donde estés escribiendo"
      >
        <span className="keys">Ctrl + Alt + V</span>
        <button type="button" className="mild">
          Cambiar
        </button>
      </Line>

      <Line says="Ocultar al perder el foco" why="El panel se va solo en cuanto tocas otra ventana">
        <Knob on={hides} says="Ocultar al perder el foco" onPress={() => setHides(!hides)} />
      </Line>
    </>
  );
}
