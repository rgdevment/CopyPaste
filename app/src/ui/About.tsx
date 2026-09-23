import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { CloudOff, Code, Gift, Key } from "./Icons";

const STARS = "https://github.com/rgdevment/CopyPaste";
const SPONSOR = "https://github.com/sponsors/rgdevment";
const COFFEE = "https://buymeacoffee.com/rgdevment";
const ALTERNATIVE = "https://alternativeto.net/software/copypaste/about/";
const RATING = "ms-windows-store://review/?ProductId=9NBJRZF3K856";

const TOOLS = [
  {
    name: "Tisty",
    hue: "#7c6cf0",
    says: "Notas, documentos y tareas, todo local y en archivos que puedes leer sin él",
    at: "https://github.com/rgdevment/Tisty",
  },
  {
    name: "LinkUnbound",
    hue: "#3fbfa6",
    says: "Elige con qué navegador se abre cada enlace, en el momento de abrirlo",
    at: "https://github.com/rgdevment/LinkUnbound",
  },
];

export default function About() {
  const [beta, setBeta] = useState(false);

  const go = (where: string) => {
    openUrl(where).catch(() => {});
  };

  return (
    <>
      <div className="brow">
        <span className="mark" aria-hidden="true">
          C
        </span>
        <span>
          <h2>CopyPaste</h2>
          <span className="line2">
            <span>3.0.0</span>
            <i />
            <span>GPL-3.0</span>
          </span>
        </span>
      </div>

      <div className="what-is">
        <p>Todo lo que copias queda a un atajo de distancia, en tu equipo y solo en tu equipo.</p>
        <p>
          Sin cuenta, sin suscripción, sin telemetría y sin servidor. Lo que copias no sale de aquí.
        </p>
        <div className="badges">
          <span className="badge">
            <Key />
            Local
          </span>
          <span className="badge">
            <Code />
            Código abierto
          </span>
          <span className="badge">
            <Gift />
            Gratis
          </span>
          <span className="badge">
            <CloudOff />
            Sin nube
          </span>
        </div>
      </div>

      <div className="newer">
        <span className="pip ok" />
        <span className="grow">
          <b>Estás en la última versión</b>
          <span>Se comprobó al abrir CopyPaste</span>
        </span>
        <button type="button" className="mild">
          Buscar ahora
        </button>
      </div>

      <label className="beta" htmlFor="beta">
        <input id="beta" type="checkbox" checked={beta} onChange={() => setBeta(!beta)} />
        <span>
          <b>Recibir versiones de prueba</b>
          <span>Llegan antes que a nadie y pueden fallar. Puedes salir cuando quieras.</span>
        </span>
      </label>

      <div className="rule">Apoyar</div>
      <p className="quiet">
        CopyPaste es gratis y lo seguirá siendo. Si te sirve, esto ayuda a que siga creciendo.
      </p>
      <div className="gives" style={{ marginTop: 10 }}>
        <button type="button" className="give" onClick={() => go(STARS)}>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="#e3b341"
              d="M8 1.2l2.1 4.3 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.2.8-4.7L1.2 6.2l4.7-.7L8 1.2z"
            />
          </svg>
          <span>
            <b>Dale una estrella</b>
            <span>github.com/rgdevment/CopyPaste</span>
          </span>
        </button>
        <button type="button" className="give" onClick={() => go(RATING)}>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="#0078d4"
              d="M2 3h12a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H6l-4 3V4a1 1 0 0 1 1-1Z"
            />
          </svg>
          <span>
            <b>Valórala en la Store</b>
            <span>Microsoft Store</span>
          </span>
        </button>
        <button type="button" className="give" onClick={() => go(SPONSOR)}>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="#db61a2"
              d="M8 14.25 6.84 13.2C2.72 9.47 0 7.01 0 4.5 0 2.42 1.57 1 3.5 1c1.1 0 2.16.51 2.84 1.32h1.32C8.34 1.51 9.4 1 10.5 1 12.43 1 14 2.42 14 4.5c0 2.51-2.72 4.97-6.84 8.7L8 14.25Z"
            />
          </svg>
          <span>
            <b>Patrocina el proyecto</b>
            <span>github.com/sponsors</span>
          </span>
        </button>
        <button type="button" className="give" onClick={() => go(COFFEE)}>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="#c8892a"
              d="M2 5h9v5a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V5Zm10 0h1.5A2.5 2.5 0 0 1 16 7.5 2.5 2.5 0 0 1 13.5 10H12V5ZM2 14h9v1H2v-1Z"
            />
          </svg>
          <span>
            <b>Invítame un café</b>
            <span>buymeacoffee.com</span>
          </span>
        </button>
      </div>

      <div className="rule">Otras herramientas</div>
      {TOOLS.map((tool) => (
        <button key={tool.name} type="button" className="tool" onClick={() => go(tool.at)}>
          <span className="ico" style={{ background: tool.hue }}>
            {tool.name.slice(0, 1)}
          </span>
          <span style={{ flex: 1, minWidth: 0 }}>
            <b>{tool.name}</b>
            <span>{tool.says}</span>
          </span>
        </button>
      ))}

      <div className="rule">Si algo va mal</div>
      <p className="quiet">
        El informe reúne el registro, la versión y los datos de tu equipo en un archivo.{" "}
        <em>No se envía a ninguna parte</em>: se guarda donde tú digas y lo adjuntas si quieres.
      </p>
      <div className="feet">
        <button type="button" className="mild">
          Guardar informe…
        </button>
        <button type="button" className="mild">
          Abrir el registro
        </button>
      </div>

      <div className="feet">
        <button type="button" className="mild" onClick={() => go(STARS)}>
          Repositorio
        </button>
        <button type="button" className="mild" onClick={() => go(ALTERNATIVE)}>
          AlternativeTo
        </button>
        <button type="button" className="mild">
          Avisos de terceros
        </button>
      </div>
    </>
  );
}
