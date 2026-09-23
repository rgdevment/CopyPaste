import { useState } from "react";
import { Band, Line } from "./Bits";

export default function History() {
  const [days, setDays] = useState("30");
  const [quota, setQuota] = useState("0");

  return (
    <>
      <h1>Historial</h1>

      <Band says="Qué se guarda" />

      <Line says="Guardar durante" why="Lo más viejo se borra solo. Lo anclado nunca caduca">
        <select
          aria-label="Guardar durante"
          value={days}
          onChange={(event) => setDays(event.target.value)}
        >
          <option value="7">7 días</option>
          <option value="30">30 días</option>
          <option value="90">90 días</option>
          <option value="0">Siempre</option>
        </select>
      </Line>

      <Line
        says="Espacio para imágenes"
        why="Al llegar al tope se van las imágenes más antiguas; el texto no se toca"
      >
        <select
          aria-label="Espacio para imágenes"
          value={quota}
          onChange={(event) => setQuota(event.target.value)}
        >
          <option value="0">Sin límite</option>
          <option value="256">256 MB</option>
          <option value="512">512 MB</option>
          <option value="1024">1 GB</option>
        </select>
      </Line>

      <Band says="Dónde vive" />

      <Line says="Carpeta de datos" why={<span className="path">Calculando…</span>}>
        <button type="button" className="mild">
          Abrir carpeta
        </button>
      </Line>

      <Line
        says="Vaciar el historial"
        why="Borra todo lo copiado, incluso lo anclado. No se puede deshacer"
      >
        <button type="button" className="grave">
          Vaciar
        </button>
      </Line>
    </>
  );
}
