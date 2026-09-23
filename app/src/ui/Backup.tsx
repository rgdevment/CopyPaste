import { Band, Line } from "./Bits";

export default function Backup() {
  return (
    <>
      <h1>Copia de seguridad</h1>

      <Band says="Tus copias" />

      <Line says="Exportar" why="Un archivo .cpbackup con todo: textos, imágenes y lo anclado">
        <button type="button" className="strong">
          Exportar…
        </button>
      </Line>

      <Line
        says="Importar"
        why="Añade lo que haya en el archivo. Nada de lo que ya tienes se pierde"
      >
        <button type="button" className="mild">
          Elegir archivo…
        </button>
      </Line>

      <Band says="La versión anterior" />

      <Line
        says="La CopyPaste 2 sigue en este equipo"
        why="No se encontró ninguna instalación anterior"
      >
        <button type="button" className="mild" disabled>
          Traer el historial…
        </button>
      </Line>
    </>
  );
}
