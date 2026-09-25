const ES = {
  railGeneral: "General",
  railHistory: "Historial",
  railBackup: "Copia de seguridad",
  railAbout: "Acerca de",
  railSections: "Secciones",

  chromeMinimise: "Minimizar",
  chromeClose: "Cerrar",

  bandLook: "Apariencia",
  bandDoes: "Comportamiento",
  tongue: "Idioma",
  tongueWhy: "El de tu equipo, salvo que elijas otro",
  tongueTheirs: "El del sistema",
  look: "Tema",
  lookWhy: "Claro, oscuro o el que use tu equipo",
  lookTheirs: "El del sistema",
  lookLight: "Claro",
  lookDark: "Oscuro",
  wake: "Arranca con la sesión",
  wakeWhy: "CopyPaste se abre al iniciar sesión y espera en la bandeja",
  wakeTheirs:
    "Windows tiene el arranque de CopyPaste desactivado. Actívalo en Configuración › Aplicaciones › Inicio.",
  keys: "Atajo del panel",
  keysWhy: "Presiónalo en cualquier parte: lo que elijas se pega donde estabas escribiendo",
  keysChange: "Cambiar",
  keysAsk: "Presiona la combinación que quieres",
  keysStop: "Dejarlo como está",
  panelTrouble:
    "El panel no está funcionando: {one}. Lo copiado no se está guardando hasta que se resuelva",
  keysTaken:
    "Otro programa ya usa esa combinación, así que el panel no se abre con ella. Elige otra cuando puedas cambiarla",
  hides: "Ocultar al hacer clic fuera",
  hidesWhy: "El panel se va solo en cuanto haces clic en otra ventana",

  bandKeeps: "Qué se guarda",
  keeps: "Conservar",
  keepsWhy: "Lo más antiguo se borra solo. Lo anclado nunca caduca",
  keepsDays: "{one} días",
  keepsForever: "Siempre",
  quota: "Espacio del historial",
  quotaWhy: "Al llegar al tope se borra lo más antiguo, sea lo que sea; lo anclado se queda",
  quotaNone: "Sin límite",
  bandWhere: "Tus datos",
  where: "Carpeta de datos",
  whereUnknown: "No se pudo averiguar",
  whereOpen: "Abrir carpeta",
  empty: "Vaciar el historial",
  emptyWhy: "Borra todo lo copiado y conserva lo que hayas anclado. No se puede deshacer",
  emptyDo: "Vaciar",
  emptySure: "¿Seguro?",
  emptyGone: "Listo, el historial quedó vacío",

  bandCopies: "Exportar e importar",
  out: "Exportar",
  outWhy: "Un archivo .cpbackup con todo: textos, imágenes y lo anclado",
  outDo: "Exportar…",
  in: "Importar",
  inWhy: "Añade lo que haya en el archivo. Nada de lo que ya tienes se pierde",
  inDo: "Elegir archivo…",
  bandFormer: "La versión anterior",
  former: "Datos de CopyPaste 2",
  formerNone: "No quedan en este equipo",
  formerLoses:
    "Sus datos se quedan donde están hasta que tú los borres. Lo que entre desde CopyPaste 2 llegará sin miniaturas, sin el texto leído de las imágenes y sin las veces que pegaste cada cosa: empezar de cero es lo recomendado.",
  formerBring: "Traer el historial…",
  formerDrop: "Borrar los datos de CopyPaste 2",

  aboutIs: "Qué es",
  aboutWhat: "Un gestor de portapapeles moderno, nativo en Windows y macOS.",
  aboutPrivacy:
    "Tu historial se queda en este equipo, siempre a mano. Sin cuentas, sin telemetría, sin suscripciones.",
  badgeLocal: "Todo local",
  badgeOpen: "Código abierto",
  badgeFree: "Gratis",
  badgeQuiet: "Sin nube",
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
  troubleWhat:
    "El informe junta en un archivo el registro de CopyPaste, su versión y qué Windows o macOS usas.",
  troubleNeverSent: "No se envía a ninguna parte",
  troubleYours: ". Se guarda donde tú elijas y solo lo adjuntas si quieres.",
  troubleReport: "Guardar informe…",
  troubleLog: "Abrir el registro",
  aboutRepo: "Repositorio",
  aboutAlternative: "AlternativeTo",
  aboutNotices: "Avisos de terceros",
  linkRefused: "No se pudo abrir {one}",
  soon: "Todavía no",
  tryAgain: "Reintentar",
  updateNone: "Todavía no busca actualizaciones por sí sola",
  updateWhen: "Las nuevas versiones llegan por donde instalaste CopyPaste",
  aboutPrivacyLink: "Privacidad",
  keysFormer: "En CopyPaste 2 era Ctrl + Alt + C",
  wakeMac: "En macOS se activa en Ajustes del Sistema › General › Elementos de inicio",
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

  bandLook: "Appearance",
  bandDoes: "Behaviour",
  tongue: "Language",
  tongueWhy: "Follows your computer, unless you pick one",
  tongueTheirs: "System",
  look: "Theme",
  lookWhy: "Light, dark, or whichever your computer uses",
  lookTheirs: "System",
  lookLight: "Light",
  lookDark: "Dark",
  wake: "Start at login",
  wakeWhy: "CopyPaste opens when you log in and waits in the tray",
  wakeTheirs:
    "Windows has CopyPaste's startup turned off. Turn it on in Settings › Apps › Startup.",
  keys: "Panel shortcut",
  keysWhy: "Press it anywhere: what you pick is pasted where you were typing",
  keysChange: "Change",
  keysAsk: "Press the combination you want",
  keysStop: "Leave it as it is",
  panelTrouble:
    "The panel is not working: {one}. Nothing you copy is being kept until this is fixed",
  keysTaken:
    "Another program already uses that combination, so the panel will not open with it. Pick another one when you can change it",
  hides: "Hide when you click elsewhere",
  hidesWhy: "The panel goes away on its own as soon as you click another window",

  bandKeeps: "What is kept",
  keeps: "Keep",
  keepsWhy: "Older items are deleted automatically. Pinned items never expire",
  keepsDays: "{one} days",
  keepsForever: "Forever",
  quota: "History size",
  quotaWhy: "When the limit is reached the oldest goes, whatever it is; pinned items stay",
  quotaNone: "No limit",
  bandWhere: "Your data",
  where: "Data folder",
  whereUnknown: "Could not be found",
  whereOpen: "Open folder",
  empty: "Empty the history",
  emptyWhy: "Deletes everything you copied and keeps whatever you pinned. It cannot be undone",
  emptyDo: "Empty",
  emptySure: "Sure?",
  emptyGone: "Done, the history is empty",

  bandCopies: "Export and import",
  out: "Export",
  outWhy: "A .cpbackup file with everything: text, images and what you pinned",
  outDo: "Export…",
  in: "Import",
  inWhy: "Adds whatever the file holds. Nothing you already have is lost",
  inDo: "Choose a file…",
  bandFormer: "The previous version",
  former: "CopyPaste 2 data",
  formerNone: "None left on this computer",
  formerLoses:
    "Its data stays where it is until you delete it. What comes across from CopyPaste 2 arrives without thumbnails, without the text read from images, and without how many times you pasted each thing: starting fresh is the recommended path.",
  formerBring: "Bring the history over…",
  formerDrop: "Delete CopyPaste 2's data",

  aboutIs: "What it is",
  aboutWhat: "A modern clipboard manager, native on Windows and macOS.",
  aboutPrivacy:
    "Your history stays on this computer, always within reach. No accounts, no telemetry, no subscriptions.",
  badgeLocal: "All local",
  badgeOpen: "Open source",
  badgeFree: "Free",
  badgeQuiet: "No cloud",
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
  troubleWhat:
    "The report puts CopyPaste's log, its version and which Windows or macOS you use into one file.",
  troubleNeverSent: "It is never sent anywhere",
  troubleYours: ". It is saved where you choose, and you attach it only if you want to.",
  troubleReport: "Save a report…",
  troubleLog: "Open the log",
  aboutRepo: "Repository",
  aboutAlternative: "AlternativeTo",
  aboutNotices: "Third-party notices",
  linkRefused: "{one} could not be opened",
  soon: "Not yet",
  tryAgain: "Try again",
  updateNone: "It does not check for updates on its own yet",
  updateWhen: "New versions arrive the way you installed CopyPaste",
  aboutPrivacyLink: "Privacy",
  keysFormer: "In CopyPaste 2 it was Ctrl + Alt + C",
  wakeMac: "On macOS, turn it on in System Settings › General › Login Items",
};

let now: Record<keyof Said, string> = ES;

export function adopt(locale: string | null) {
  const asked = locale ?? navigator.language;
  const english = asked.toLowerCase().startsWith("en");
  now = english ? EN : ES;
  document.documentElement.lang = english ? "en" : "es";
}

export function t(key: keyof Said) {
  return now[key];
}

export function fill(key: keyof Said, one: string) {
  return now[key].replace("{one}", one);
}
