import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Band, Line } from "./Bits";

type Former = { path: string; bytes: number };

function weighed(bytes: number) {
  const mb = bytes / (1024 * 1024);
  return mb >= 1 ? `${mb.toFixed(1)} MB` : `${Math.round(bytes / 1024)} KB`;
}

export default function Backup() {
  const [former, setFormer] = useState<Former | null>(null);

  useEffect(() => {
    invoke<Former | null>("former")
      .then(setFormer)
      .catch(() => setFormer(null));
  }, []);

  return (
    <>
      <h1>Copia de seguridad</h1>

      <Band says="Tus copias" />

      <Line says="Exportar" why="Un archivo .cpbackup con todo: textos, imágenes y lo anclado">
        <button type="button" className="strong" disabled>
          Exportar…
        </button>
      </Line>

      <Line
        says="Importar"
        why="Añade lo que haya en el archivo. Nada de lo que ya tienes se pierde"
      >
        <button type="button" className="mild" disabled>
          Elegir archivo…
        </button>
      </Line>

      <Band says="La versión anterior" />

      {former ? (
        <Line
          says="CopyPaste 2 sigue en este equipo"
          why={<span className="path">{`${former.path} · ${weighed(former.bytes)}`}</span>}
          more={
            <div className="said">
              Sus datos se quedan donde están hasta que tú los borres. Lo que entre desde CopyPaste
              2 llegará sin miniaturas, sin el texto leído de las imágenes y sin las veces que
              pegaste cada cosa: empezar de cero es lo recomendado.
            </div>
          }
        >
          <button type="button" className="mild" disabled>
            Traer el historial…
          </button>
          <button type="button" className="grave" disabled>
            Eliminar sus datos
          </button>
        </Line>
      ) : (
        <Line
          says="CopyPaste 2 sigue en este equipo"
          why="No se encontró ninguna instalación anterior"
        />
      )}
    </>
  );
}
