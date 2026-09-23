import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { type Kept, whereItLives } from "../core";
import { Band, Line } from "./Bits";

export default function History({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => void;
}) {
  const [where, setWhere] = useState<string | null>(null);

  useEffect(() => {
    whereItLives()
      .then(setWhere)
      .catch(() => setWhere(null));
  }, []);

  return (
    <>
      <h1>Historial</h1>

      <Band says="Qué se guarda" />

      <Line says="Guardar durante" why="Lo más viejo se borra solo. Lo anclado nunca caduca">
        <select
          aria-label="Guardar durante"
          value={String(kept["keeps-days"] ?? 0)}
          onChange={(event) => change({ "keeps-days": Number(event.target.value) || null })}
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
          value={String(kept["images-quota-mb"] ?? 0)}
          onChange={(event) => change({ "images-quota-mb": Number(event.target.value) || null })}
        >
          <option value="0">Sin límite</option>
          <option value="256">256 MB</option>
          <option value="512">512 MB</option>
          <option value="1024">1 GB</option>
        </select>
      </Line>

      <Band says="Dónde vive" />

      <Line
        says="Carpeta de datos"
        why={<span className="path">{where ?? "No se pudo averiguar"}</span>}
      >
        <button
          type="button"
          className="mild"
          disabled={!where}
          onClick={() => {
            if (where) void revealItemInDir(where).catch(() => {});
          }}
        >
          Abrir carpeta
        </button>
      </Line>

      <Line
        says="Vaciar el historial"
        why="Borra todo lo copiado, incluso lo anclado. No se puede deshacer"
      >
        <button type="button" className="grave" disabled>
          Vaciar
        </button>
      </Line>
    </>
  );
}
