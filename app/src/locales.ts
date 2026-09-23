const ES = {
  railGeneral: "General",
  railHistory: "Historial",
  railBackup: "Copia de seguridad",
  railAbout: "Acerca de",
  railSections: "Secciones",

  chromeMinimise: "Minimizar",
  chromeClose: "Cerrar",

  bandWindow: "Ventana",
  tongue: "Idioma",
  tongueWhy: "El de tu equipo, salvo que elijas otro",
  tongueTheirs: "El del sistema",
  look: "Tema",
  lookWhy: "Claro, oscuro, o el que use tu equipo",
  lookTheirs: "El del sistema",
  lookLight: "Claro",
  lookDark: "Oscuro",
  wake: "Arranca con la sesión",
  wakeWhy: "CopyPaste se abre al iniciar tu equipo y espera en la bandeja",
  wakeTheirs:
    "Windows tiene desactivado el arranque de CopyPaste. Actívalo en Inicio, dentro del Administrador de tareas.",
  keys: "Atajo del panel",
  keysWhy: "Presiónalo en cualquier parte y el panel aparece donde estés escribiendo",
  keysChange: "Cambiar",
  hides: "Ocultar al perder el foco",
  hidesWhy: "El panel se va solo en cuanto tocas otra ventana",

  bandKeeps: "Qué se guarda",
  keeps: "Guardar durante",
  keepsWhy: "Lo más viejo se borra solo. Lo anclado nunca caduca",
  keepsDays: "{one} días",
  keepsForever: "Siempre",
  quota: "Espacio para imágenes",
  quotaWhy: "Al llegar al tope se van las imágenes más antiguas; el texto no se toca",
  quotaNone: "Sin límite",
  bandWhere: "Dónde vive",
  where: "Carpeta de datos",
  whereUnknown: "No se pudo averiguar",
  whereOpen: "Abrir carpeta",
  empty: "Vaciar el historial",
  emptyWhy: "Borra todo lo copiado, incluso lo anclado. No se puede deshacer",
  emptyDo: "Vaciar",

  bandCopies: "Tus copias",
  out: "Exportar",
  outWhy: "Un archivo .cpbackup con todo: textos, imágenes y lo anclado",
  outDo: "Exportar…",
  in: "Importar",
  inWhy: "Añade lo que haya en el archivo. Nada de lo que ya tienes se pierde",
  inDo: "Elegir archivo…",
  bandFormer: "La versión anterior",
  former: "CopyPaste 2 sigue en este equipo",
  formerNone: "No se encontró ninguna instalación anterior",
  formerLoses:
    "Sus datos se quedan donde están hasta que tú los borres. Lo que entre desde CopyPaste 2 llegará sin miniaturas, sin el texto leído de las imágenes y sin las veces que pegaste cada cosa: empezar de cero es lo recomendado.",
  formerBring: "Traer el historial…",
  formerDrop: "Eliminar sus datos",

  aboutWhat: "Un gestor de portapapeles moderno, nativo en Windows y macOS.",
  aboutPrivacy:
    "Todo local — tu historial, siempre a mano. Sin cuentas, sin telemetría, sin suscripciones.",
  badgeLocal: "Todo local",
  badgeOpen: "Código abierto",
  badgeFree: "Gratis",
  badgeQuiet: "Sin nube",
  updateNone: "Estás en la última versión",
  updateWhen: "Se comprobó al abrir CopyPaste",
  updateLook: "Buscar ahora",
  betaTake: "Recibir versiones de prueba",
  betaWarns: "Llegan antes que a nadie y pueden fallar. Puedes salir cuando quieras.",
  supportTitle: "Apoyar",
  supportWhy:
    "CopyPaste es gratis y lo seguirá siendo. Si te sirve, esto ayuda a que siga creciendo.",
  supportStar: "Dale una estrella",
  supportRate: "Valórala en la Store",
  supportSponsor: "Patrocina el proyecto",
  supportCoffee: "Invítame un café",
  otherTools: "Otras herramientas",
  toolTisty: "Notas, documentos y tareas, todo local y en archivos que puedes leer sin él",
  toolLinkUnbound: "Elige con qué navegador se abre cada enlace, en el momento de abrirlo",
  troubleTitle: "Si algo va mal",
  troubleWhat: "El informe reúne el registro, la versión y los datos de tu equipo en un archivo.",
  troubleNeverSent: "No se envía a ninguna parte",
  troubleYours: ": se guarda donde tú digas y lo adjuntas si quieres.",
  troubleReport: "Guardar informe…",
  troubleLog: "Abrir el registro",
  aboutRepo: "Repositorio",
  aboutAlternative: "AlternativeTo",
  aboutNotices: "Avisos de terceros",
} as const;

type Said = typeof ES;

const EN: Record<keyof Said, string> = {
  railGeneral: "General",
  railHistory: "History",
  railBackup: "Backup",
  railAbout: "About",
  railSections: "Sections",

  chromeMinimise: "Minimise",
  chromeClose: "Close",

  bandWindow: "Window",
  tongue: "Language",
  tongueWhy: "Your computer's, unless you pick another",
  tongueTheirs: "System",
  look: "Theme",
  lookWhy: "Light, dark, or whichever your computer uses",
  lookTheirs: "System",
  lookLight: "Light",
  lookDark: "Dark",
  wake: "Start with the session",
  wakeWhy: "CopyPaste opens when your computer starts and waits in the tray",
  wakeTheirs:
    "Windows has CopyPaste's startup turned off. Turn it back on under Startup, in Task Manager.",
  keys: "Panel shortcut",
  keysWhy: "Press it anywhere and the panel appears where you are typing",
  keysChange: "Change",
  hides: "Hide when it loses focus",
  hidesWhy: "The panel leaves on its own as soon as you touch another window",

  bandKeeps: "What is kept",
  keeps: "Keep for",
  keepsWhy: "The oldest goes on its own. Pinned items never expire",
  keepsDays: "{one} days",
  keepsForever: "Forever",
  quota: "Room for images",
  quotaWhy: "At the limit the oldest images go; text is left alone",
  quotaNone: "No limit",
  bandWhere: "Where it lives",
  where: "Data folder",
  whereUnknown: "Could not be found",
  whereOpen: "Open folder",
  empty: "Empty the history",
  emptyWhy: "Deletes everything you copied, pinned included. It cannot be undone",
  emptyDo: "Empty",

  bandCopies: "Your copies",
  out: "Export",
  outWhy: "A .cpbackup file with everything: text, images and what you pinned",
  outDo: "Export…",
  in: "Import",
  inWhy: "Adds whatever the file holds. Nothing you already have is lost",
  inDo: "Choose a file…",
  bandFormer: "The previous version",
  former: "CopyPaste 2 is still on this computer",
  formerNone: "No earlier installation was found",
  formerLoses:
    "Its data stays where it is until you delete it. What comes across from CopyPaste 2 arrives without thumbnails, without the text read from images, and without how many times you pasted each thing: starting fresh is the recommended path.",
  formerBring: "Bring the history over…",
  formerDrop: "Delete its data",

  aboutWhat: "A modern clipboard manager, native on Windows and macOS.",
  aboutPrivacy:
    "All local — your history, always within reach. No accounts, no telemetry, no subscriptions.",
  badgeLocal: "All local",
  badgeOpen: "Open source",
  badgeFree: "Free",
  badgeQuiet: "No cloud",
  updateNone: "You are on the latest version",
  updateWhen: "Checked when CopyPaste opened",
  updateLook: "Check now",
  betaTake: "Get test versions",
  betaWarns:
    "They arrive before anyone else's and they can break. You can leave whenever you like.",
  supportTitle: "Support",
  supportWhy: "CopyPaste is free and will stay free. If it helps you, this helps it keep growing.",
  supportStar: "Give it a star",
  supportRate: "Rate it on the Store",
  supportSponsor: "Sponsor the project",
  supportCoffee: "Buy me a coffee",
  otherTools: "Other tools",
  toolTisty: "Notes, documents and tasks, all local and in files you can read without it",
  toolLinkUnbound: "Choose which browser opens each link, at the moment you open it",
  troubleTitle: "If something goes wrong",
  troubleWhat: "The report gathers the log, the version and your computer's details into one file.",
  troubleNeverSent: "It is never sent anywhere",
  troubleYours: ": it is saved where you say, and you attach it if you want to.",
  troubleReport: "Save a report…",
  troubleLog: "Open the log",
  aboutRepo: "Repository",
  aboutAlternative: "AlternativeTo",
  aboutNotices: "Third-party notices",
};

const SAID: Record<string, Record<keyof Said, string>> = { es: ES, en: EN };

let now: Record<keyof Said, string> = ES;

export function adopt(locale: string | null) {
  const asked = locale ?? navigator.language;
  now = asked.toLowerCase().startsWith("en") ? EN : ES;
}

export function t(key: keyof Said) {
  return now[key];
}

export function fill(key: keyof Said, one: string) {
  return now[key].replace("{one}", one);
}

export function tongues() {
  return Object.keys(SAID);
}
