const ES = {
  railGeneral: "General",
  railHistory: "Historial",
  railKeys: "Atajos de teclado",
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
  wakeManaged:
    "Windows gestiona el arranque de CopyPaste. Cámbialo en Configuración › Aplicaciones › Inicio.",
  keys: "Atajo del panel",
  keysWhy: "Presiónalo en cualquier parte: lo que elijas se pega donde estabas escribiendo",
  keysChange: "Cambiar",
  keysAsk: "Presiona la combinación que quieres",
  keysStop: "Dejarlo como está",
  panelTrouble:
    "El panel no está funcionando: {one}. Lo copiado no se está guardando hasta que se resuelva",
  keysTaken:
    "Otro programa ya usa esa combinación, así que el panel no se abre con ella. Elige otra cuando puedas cambiarla",
  keysFree: "Estas están libres ahora mismo:",
  bandOpens: "ABRIR EL PANEL",
  keysTable: "Lo que responde el panel",
  keysTableWhy: "Estas teclas funcionan mientras el panel está abierto",
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
  formerHas: "{one} guardados en CopyPaste 2",
  formerKeepsStyles: "{one} conservan sus estilos, los que la 2 alcanzó a guardar",
  formerLosesPlain:
    "El resto llega en plano: de esos la 2 guardó solo el texto, así que lo que pegaba con estilos ya no los tiene",
  formerLosesPictures: "{one} imágenes ya no están en el disco y llegarán sin su contenido",
  formerLosesKept:
    "{one} son más antiguos que el tiempo que guardas: los que cruces se irán al llegar. Sube el límite en Ajustes si quieres conservarlos",
  formerKeepsSecrets:
    "La 2 guardó cosas que hoy no se capturarían, como lo que un gestor de contraseñas marca en privado. Lo que ya guardó, cruza igual",
  formerPanelRests: "El panel se detiene mientras cruza el historial y vuelve solo al terminar",
  formerKeeps:
    "Cruzan las fechas, lo anclado, las etiquetas, el color, de qué aplicación vino cada cosa y cuántas veces la pegaste",
  formerDo: "Traer el historial",
  formerBringing: "Trayéndolo…",
  formerCrossing: "Trayendo {one}…",
  failedLook: "No se pudo leer lo que dejó CopyPaste 2",
  failedOut: "No se pudo escribir la copia",
  failedIn: "No se pudo traer lo que había en el archivo",
  failedCross: "No se pudo traer el historial de CopyPaste 2",
  failedDrop: "No se pudieron borrar los datos de CopyPaste 2",
  failedEmpty: "No se pudo vaciar el historial",
  inTheLog: "el registro dice por qué",
  formerCame: "Llegaron {one}",
  formerCameOn: "Ya lo trajiste el {one}",
  formerStillHere: "{one} de CopyPaste 2 siguen aquí",
  formerGoneStays: "Lo que borraste desde entonces no vuelve",
  formerAgainWhy:
    "Volver a revisar solo trae algo si subes el tiempo que guardas o el espacio que diste a las imágenes",
  formerAgain: "Volver a revisar",
  formerCameAlready: "{one} ya estaban",
  formerCameRefused: "{one} no se pudieron traer",
  formerCameFlat: "{one} sin su imagen",
  formerCameSwept: "{one} se fueron por el tiempo que guardas",
  formerCameCrowded: "{one} se fueron por el espacio que reservaste para las imágenes",
  formerUnreadable: "No se pudo leer: {one}",
  formerStays: "Nada de CopyPaste 2 se toca ni se borra: se lee y se queda donde está",
  formerDrop: "Borrar los datos de CopyPaste 2",
  formerDropSure: "Sí, borrarlos",
  formerDropWhy:
    "Se borran su historial, sus imágenes y sus ajustes. Los archivos que copiaste no se tocan: esos son tuyos",
  formerDropGone: "Borrado: {one}",
  formerDropFiles: "{one} archivos",

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
  toolLinkUnboundHere: "Instalado · tus enlaces salen por él",
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
  noticesRefused: "No se pudieron leer los avisos de terceros",
  aboutLicences: "Textos de las licencias",
  licencesRefused: "No se pudieron leer los textos de las licencias",
  linkRefused: "No se pudo abrir {one}",
  soon: "Todavía no",
  tryAgain: "Reintentar",
  updateNone: "Estás en la última versión",
  updateWhen: "Se mira una vez al día, y cuando tú lo pidas",
  updateThere: "La versión {one} ya está disponible",
  updateTake: "Se descarga, se instala y CopyPaste se reinicia solo",
  updateMove: "Para instalarla, arrastra CopyPaste a Aplicaciones y ábrelo desde ahí",
  updateDo: "Actualizar",
  updateGetting: "Descargando…",
  updateStore: "La Microsoft Store se encarga",
  updateStoreWhy: "Ella instala las versiones nuevas cuando están certificadas",
  updateOffline: "No se pudo comprobar ahora. Revisa tu conexión y vuelve a intentarlo.",
  updateUnreadable: "La información de versiones no se pudo leer. Vuelve a intentarlo más tarde.",
  updatePublishing: "Esta versión se está publicando. Vuelve a intentarlo en unos minutos.",
  updateGone: "Esa versión ya no está disponible. Busca de nuevo.",
  updateFailed:
    "No se pudo actualizar. Vuelve a intentarlo; si sigue fallando, descárgala desde la web.",
  updateBusy: "Ya hay una actualización en marcha.",
  aboutPrivacyLink: "Privacidad",
  keysFormer: "En CopyPaste 2 era Ctrl + Alt + C",
  wakeTheirsMac: "Aprueba CopyPaste en Ajustes del Sistema › General › Ítems de inicio",
  trust: "Permiso para pegar",
  trustWhy: "macOS pide tu permiso para que CopyPaste escriba en la ventana donde estabas",
  trustGranted: "Concedido: lo que elijas se pega solo",
  trustMissing: "Sin él, lo elegido queda en el portapapeles y lo pegas tú",
  trustAsk: "Conceder",
  trustOpen: "Abrir Ajustes del Sistema",
  clipboardDenied:
    "macOS no deja que CopyPaste lea el portapapeles: actívalo en Ajustes del Sistema › Privacidad y seguridad › Pegar desde otras apps",
  clipboardAsks:
    "macOS puede preguntarte si CopyPaste puede leer el portapapeles: responde «Permitir» para que guarde tus copias",
  trustSecure: "Hay un campo de contraseña abierto: mientras dure, nadie puede pegar por ti",
  outDone: "Guardado: {one}",
  outMissing: "faltaron {one} sin su contenido",
  outPlain:
    "El archivo lleva tu historial sin cifrar: guárdalo donde guardarías el historial mismo",
  inRefused: "{one} no se pudieron traer",
  inElsewhere: "viene del otro sistema: puede que algo no se pegue igual",
  inDone: "Llegaron {one}",
  inAlready: "{one} ya estaban y no se duplicaron",
  inNothing: "Todo lo del archivo ya estaba aquí",
  busy: "Un momento…",
  itemOne: "1 elemento",
  itemMany: "{one} elementos",

  welcomeHello: "Tu portapapeles, con memoria",
  welcomeHelloWhy:
    "CopyPaste guarda lo que copias (textos, imágenes, enlaces y archivos) para que lo tengas a mano cuando lo necesites. Te mostramos lo básico en un minuto.",
  welcomeLocal: "Todo queda en tu equipo",
  welcomeNoAccount: "Sin cuenta",
  welcomeNoTracking: "Sin rastreo",
  welcomeSkip: "Omitir",
  welcomeStart: "Comenzar",
  welcomeNext: "Siguiente",
  welcomeSkipStep: "Omitir este paso",
  welcomeTryAgain: "Probar de nuevo",
  welcomeStep: "Paso {one}",
  welcomeKeysOver: "1 · El atajo",
  welcomeKeysAloneOver: "El atajo",
  welcomeKeysTitle: "Pruébalo ahora",
  welcomeKeysWhy: "Desde cualquier app, esta combinación abre tu historial.",
  welcomeKeysWaiting: "Esperando que lo pruebes…",
  welcomeKeysWaitingWhy:
    "Cuando se abra el panel, seguimos. Para cerrarlo, presiona Esc o haz clic afuera.",
  welcomeKeysDone: "¡Listo! Así se abre",
  welcomeKeysDoneWhy: "Puedes cambiarlo cuando quieras en Ajustes › Atajos de teclado.",
  welcomeKeysTaken: "Ese atajo ya está en uso",
  welcomeKeysTakenWhy:
    "Otra app usa {one} en este equipo. Elige uno de estos, que están libres, y lo probamos.",
  welcomeKeysTakenAlone:
    "Otra app usa {one} en este equipo. Elige otro en Ajustes › Atajos de teclado.",
  welcomeKeysOwn: "También puedes definir el tuyo en Ajustes › Atajos de teclado.",
  welcomeWhereOver: "2 · Dónde encontrarlo",
  welcomeWhereTitle: "Junto al reloj",
  welcomeWhereWhy:
    "CopyPaste vive en la bandeja del sistema, abajo a la derecha. Si no lo ves, revisa la flecha de íconos ocultos; puedes arrastrarlo a la barra para tenerlo siempre visible.",
  welcomeWhereTitleMac: "En la barra de menús",
  welcomeWhereWhyMac:
    "CopyPaste vive en la barra de menús, arriba a la derecha. Si tienes muchos íconos, puede quedar oculto detrás de la muesca de la pantalla.",
  welcomeClick: "Clic",
  welcomeClickDoes: "abre el panel",
  welcomeRightClick: "Clic derecho",
  welcomeRightClickDoes: "Ajustes y salir",
  welcomeUseOver: "3 · Cómo usarlo",
  welcomeUseTitle: "Lo esencial",
  welcomeUseWhy: "Con esto tienes casi todo. El resto está en Ajustes.",
  welcomeUseType: "Escribe",
  welcomeUseTypeDoes: "busca en tu historial al instante",
  welcomeUseEnterDoes: "pega lo que elegiste",
  welcomeUsePlainDoes: "pega solo el texto, sin formato",
  welcomeUseSettingsDoes: "abre Ajustes",
  welcomeAllKeys: "Ver todos los atajos",
  welcomeOpenSettings: "Abrir Ajustes",
  welcomeDone: "Listo",
  welcomeFormerOver: "Ya estás en la 3.0",
  welcomeFormerTitle: "Tu historial anterior te espera",
  welcomeFormerWhy:
    "Encontramos lo que guardaste en la versión 2. Lo importamos una sola vez y la versión anterior queda intacta.",
  welcomeFormerItems: "elementos",
  welcomeFormerLabelled: "con nombre o color",
  welcomeFormerPictures: "imágenes",
  welcomeFormerLater:
    "Si prefieres empezar de cero, puedes importarlo después desde Ajustes › Copia de seguridad.",
  welcomeFormerNotNow: "Ahora no",
  welcomeFormerBring: "Importar historial",
  welcomeFormerBringing: "Importando…",
  welcomeFormerCame: "Listo: {one} ya están en tu historial.",
  welcomeFormerFailed: "No se pudo importar. Puedes intentarlo desde Ajustes › Copia de seguridad.",
  welcomeTrustOver: "Antes de comenzar",
  welcomeTrustTitle: "Un permiso para poder pegar",
  welcomeTrustWhy:
    "Para pegar en la app donde estabas, macOS necesita que autorices a CopyPaste. No lee lo que escribes: solo envía ⌘V cuando eliges algo.",
  welcomeTrustStepOne:
    "1 · Abrimos Configuración del Sistema › Privacidad y seguridad › Accesibilidad",
  welcomeTrustStepTwo: "2 · Activa CopyPaste en la lista",
  welcomeTrustStepThree: "3 · Vuelve aquí: seguimos automáticamente",
  welcomeTrustWaiting: "Esperando el permiso…",
  welcomeTrustWaitingWhy:
    "Activa CopyPaste en Accesibilidad. Si ya aparecía activado y no responde, quítalo con − y agrégalo de nuevo.",
  welcomeTrustDone: "Listo, ya puede pegar",
  welcomeTrustAsk: "Dar permiso",
  welcomeTrustLater: "Ahora no",
  welcomeNewsOver: "Novedades",
  welcomeNewsTitle: "Hay novedades en CopyPaste",
  welcomeNewsAll: "Ver todas las versiones",
  welcomeNewsOk: "Entendido",
  welcomeShowAgain: "Ver la bienvenida de nuevo",
} as const;

type Said = typeof ES;

const EN: Record<keyof Said, string> = {
  railGeneral: "General",
  railHistory: "History",
  railKeys: "Keyboard shortcuts",
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
  wakeManaged: "Windows manages the startup of CopyPaste. Change it in Settings › Apps › Startup.",
  keys: "Panel shortcut",
  keysWhy: "Press it anywhere: what you pick is pasted where you were typing",
  keysChange: "Change",
  keysAsk: "Press the combination you want",
  keysStop: "Leave it as it is",
  panelTrouble:
    "The panel is not working: {one}. Nothing you copy is being kept until this is fixed",
  keysTaken:
    "Another program already uses that combination, so the panel will not open with it. Pick another one when you can change it",
  keysFree: "These are free right now:",
  bandOpens: "OPENING THE PANEL",
  keysTable: "What the panel answers to",
  keysTableWhy: "These keys work while the panel is open",
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
  formerHas: "{one} kept in CopyPaste 2",
  formerKeepsStyles: "{one} keep their styling, the ones the 2 did store",
  formerLosesPlain:
    "The rest arrive plain: for those the 2 kept the text alone, so what used to paste with styling no longer has it",
  formerLosesPictures: "{one} pictures are no longer on disk and will arrive without their content",
  formerLosesKept:
    "{one} are older than the time you keep: whichever cross will go as they land. Raise the limit in Settings if you want to keep them",
  formerKeepsSecrets:
    "The 2 stored things that would not be captured today, such as what a password manager marks as private. What it already stored comes over all the same",
  formerPanelRests:
    "The panel rests while the history crosses and comes back on its own when it is done",
  formerKeeps:
    "Dates, what was pinned, labels, colour, which app each one came from and how many times you pasted it all cross over",
  formerDo: "Bring the history over",
  formerBringing: "Bringing it over…",
  formerCrossing: "Bringing {one} over…",
  failedLook: "What CopyPaste 2 left behind could not be read",
  failedOut: "The copy could not be written",
  failedIn: "What was in the file could not be brought in",
  failedCross: "The CopyPaste 2 history could not be brought over",
  failedDrop: "The CopyPaste 2 data could not be deleted",
  failedEmpty: "The history could not be emptied",
  inTheLog: "the log says why",
  formerCame: "{one} came over",
  formerCameOn: "You brought it over on {one}",
  formerStillHere: "{one} from CopyPaste 2 are still here",
  formerGoneStays: "What you deleted since does not come back",
  formerAgainWhy:
    "Looking again only brings something if you raise the time you keep or the space you set aside for pictures",
  formerAgain: "Look again",
  formerCameAlready: "{one} were already here",
  formerCameRefused: "{one} could not be brought",
  formerCameFlat: "{one} without their picture",
  formerCameSwept: "{one} went by the time you keep",
  formerCameCrowded: "{one} went by the space you set aside for pictures",
  formerUnreadable: "It could not be read: {one}",
  formerStays: "Nothing of CopyPaste 2 is touched or deleted: it is read and left where it is",
  formerDrop: "Delete CopyPaste 2's data",
  formerDropSure: "Yes, delete it",
  formerDropWhy:
    "Its history, its pictures and its settings go. The files you copied are untouched: those are yours",
  formerDropGone: "Deleted: {one}",
  formerDropFiles: "{one} files",

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
  toolLinkUnboundHere: "Installed · your links open through it",
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
  noticesRefused: "The third-party notices could not be read",
  aboutLicences: "Licence texts",
  licencesRefused: "The licence texts could not be read",
  linkRefused: "{one} could not be opened",
  soon: "Not yet",
  tryAgain: "Try again",
  updateNone: "You are on the latest version",
  updateWhen: "Looked up once a day, and whenever you ask",
  updateThere: "Version {one} is out",
  updateTake: "It downloads, installs and CopyPaste restarts on its own",
  updateMove: "To install it, drag CopyPaste to Applications and open it from there",
  updateDo: "Update",
  updateGetting: "Downloading…",
  updateStore: "The Microsoft Store takes care of it",
  updateStoreWhy: "It installs new versions once they are certified",
  updateOffline: "Could not check right now. Check your connection and try again.",
  updateUnreadable: "The version information could not be read. Try again later.",
  updatePublishing: "This version is still being published. Try again in a few minutes.",
  updateGone: "That version is no longer available. Check again.",
  updateFailed:
    "The update did not go through. Try again; if it keeps failing, download it from the website.",
  updateBusy: "An update is already on its way.",
  aboutPrivacyLink: "Privacy",
  keysFormer: "In CopyPaste 2 it was Ctrl + Alt + C",
  wakeTheirsMac: "Allow CopyPaste in System Settings › General › Login Items",
  trust: "Permission to paste",
  trustWhy: "macOS asks for your permission before CopyPaste types into the window you were in",
  trustGranted: "Granted: what you pick is pasted for you",
  trustMissing: "Without it, what you pick stays on the clipboard for you to paste",
  trustAsk: "Grant",
  trustOpen: "Open System Settings",
  clipboardDenied:
    "macOS does not let CopyPaste read the clipboard: allow it in System Settings › Privacy & Security › Paste from Other Apps",
  clipboardAsks:
    "macOS may ask whether CopyPaste can read the clipboard: answer “Allow” so it can keep your copies",
  trustSecure: "A password field is open: for as long as it is, nobody can paste for you",
  outDone: "Saved: {one}",
  outMissing: "{one} travelled without their contents",
  outPlain:
    "The file holds your history unencrypted: keep it where you would keep the history itself",
  inRefused: "{one} could not be brought in",
  inElsewhere: "it comes from the other system: some of it may not paste the same",
  inDone: "Brought in: {one}",
  inAlready: "{one} were already here and were not duplicated",
  inNothing: "Everything in the file was already here",
  busy: "One moment…",
  itemOne: "1 item",
  itemMany: "{one} items",

  welcomeHello: "Your clipboard, with a memory",
  welcomeHelloWhy:
    "CopyPaste keeps what you copy (text, images, links and files) so it is within reach when you need it. Here are the basics in a minute.",
  welcomeLocal: "Stays on your computer",
  welcomeNoAccount: "No account",
  welcomeNoTracking: "No tracking",
  welcomeSkip: "Skip",
  welcomeStart: "Get started",
  welcomeNext: "Next",
  welcomeSkipStep: "Skip this step",
  welcomeTryAgain: "Try again",
  welcomeStep: "Step {one}",
  welcomeKeysOver: "1 · The shortcut",
  welcomeKeysAloneOver: "The shortcut",
  welcomeKeysTitle: "Try it now",
  welcomeKeysWhy: "From any app, this combination opens your history.",
  welcomeKeysWaiting: "Waiting for you to try it…",
  welcomeKeysWaitingWhy:
    "Once the panel opens, we move on. To close it, press Esc or click outside.",
  welcomeKeysDone: "That is how it opens",
  welcomeKeysDoneWhy: "You can change it any time in Settings › Keyboard shortcuts.",
  welcomeKeysTaken: "That shortcut is taken",
  welcomeKeysTakenWhy:
    "Another app uses {one} on this computer. Pick one of these, which are free, and we will try it.",
  welcomeKeysTakenAlone:
    "Another app uses {one} on this computer. Pick another one in Settings › Keyboard shortcuts.",
  welcomeKeysOwn: "You can also set your own in Settings › Keyboard shortcuts.",
  welcomeWhereOver: "2 · Where to find it",
  welcomeWhereTitle: "Next to the clock",
  welcomeWhereWhy:
    "CopyPaste lives in the system tray, bottom right. If you do not see it, check the hidden icons arrow; drag it onto the taskbar to keep it in sight.",
  welcomeWhereTitleMac: "In the menu bar",
  welcomeWhereWhyMac:
    "CopyPaste lives in the menu bar, top right. With many icons, it may hide behind the notch.",
  welcomeClick: "Click",
  welcomeClickDoes: "opens the panel",
  welcomeRightClick: "Right-click",
  welcomeRightClickDoes: "Settings and quit",
  welcomeUseOver: "3 · How to use it",
  welcomeUseTitle: "The essentials",
  welcomeUseWhy: "That covers almost everything. The rest is in Settings.",
  welcomeUseType: "Type",
  welcomeUseTypeDoes: "searches your history instantly",
  welcomeUseEnterDoes: "pastes what you picked",
  welcomeUsePlainDoes: "pastes just the text, no formatting",
  welcomeUseSettingsDoes: "opens Settings",
  welcomeAllKeys: "See all shortcuts",
  welcomeOpenSettings: "Open Settings",
  welcomeDone: "Done",
  welcomeFormerOver: "You are on 3.0",
  welcomeFormerTitle: "Your previous history is waiting",
  welcomeFormerWhy:
    "We found what you saved with version 2. We import it once, and the previous version stays untouched.",
  welcomeFormerItems: "items",
  welcomeFormerLabelled: "with a name or colour",
  welcomeFormerPictures: "images",
  welcomeFormerLater:
    "If you would rather start fresh, you can import it later from Settings › Backup.",
  welcomeFormerNotNow: "Not now",
  welcomeFormerBring: "Import history",
  welcomeFormerBringing: "Importing…",
  welcomeFormerCame: "Done: {one} are now in your history.",
  welcomeFormerFailed: "It could not be imported. You can try again from Settings › Backup.",
  welcomeTrustOver: "Before you start",
  welcomeTrustTitle: "One permission, so it can paste",
  welcomeTrustWhy:
    "To paste into the app you were using, macOS needs you to allow CopyPaste. It does not read what you type: it only sends ⌘V when you pick something.",
  welcomeTrustStepOne: "1 · We open System Settings › Privacy & Security › Accessibility",
  welcomeTrustStepTwo: "2 · Turn on CopyPaste in the list",
  welcomeTrustStepThree: "3 · Come back here: we carry on by ourselves",
  welcomeTrustWaiting: "Waiting for the permission…",
  welcomeTrustWaitingWhy:
    "Turn on CopyPaste in Accessibility. If it was already on and does not respond, remove it with − and add it again.",
  welcomeTrustDone: "All set, it can paste now",
  welcomeTrustAsk: "Allow",
  welcomeTrustLater: "Not now",
  welcomeNewsOver: "What is new",
  welcomeNewsTitle: "CopyPaste has been updated",
  welcomeNewsAll: "See all versions",
  welcomeNewsOk: "Got it",
  welcomeShowAgain: "Show the welcome again",
};

export type Binding = { id: string; keys: string; said: string; does: string };

type Row = {
  id: string;
  keys: string;
  keysEn?: string;
  mac?: string;
  es: string;
  en: string;
};

const PANEL_ROWS = [
  {
    id: "paste",
    keys: "Enter",
    mac: "⏎",
    es: "pegar lo seleccionado",
    en: "paste what is selected",
  },
  {
    id: "plain",
    keys: "Shift + Enter",
    mac: "⇧⏎",
    es: "pegar en plano",
    en: "paste as plain text",
  },
  {
    id: "forms",
    keys: "Alt + Enter  ·  Ctrl + Enter",
    mac: "⌥⏎  ·  ⌘⏎",
    es: "pegar como…",
    en: "paste as…",
  },
  {
    id: "move",
    keys: "Flechas",
    keysEn: "Arrows",
    es: "moverse por la lista",
    en: "move through the list",
  },
  {
    id: "open-card",
    keys: "Clic",
    keysEn: "Click",
    es: "abrir la tarjeta; otro clic la cierra",
    en: "open the card; another click closes it",
  },
  {
    id: "paste-card",
    keys: "Doble clic",
    keysEn: "Double click",
    es: "pegar esa tarjeta",
    en: "paste that card",
  },
  {
    id: "cycle",
    keys: "Tab  ·  Shift + Tab",
    mac: "⇥  ·  ⇧⇥",
    es: "recorrer los filtros",
    en: "step through the filters",
  },
  {
    id: "layers",
    keys: "Varios",
    keysEn: "Many",
    es: "el botón de capas: los clics suman tipos en vez de cambiarlos",
    en: "the layers button: clicks add kinds instead of swapping them",
  },
  {
    id: "tags",
    keys: "#imagen  ·  #carpeta",
    keysEn: "#image  ·  #folder",
    es: "filtrar por tipo desde el buscador",
    en: "filter by kind from the search box",
  },
  {
    id: "drop-tag",
    keys: "Retroceso",
    keysEn: "Backspace",
    mac: "⌫",
    es: "quitar la última etiqueta",
    en: "drop the last tag",
  },
  {
    id: "remove",
    keys: "Supr",
    keysEn: "Delete",
    mac: "⌘⌫",
    es: "borrar la seleccionada",
    en: "delete the selected one",
  },
  { id: "pin", keys: "Ctrl + P", mac: "⌘P", es: "anclar o desanclar", en: "pin or unpin" },
  {
    id: "name",
    keys: "F2  ·  Ctrl + E",
    mac: "⌘E",
    es: "ponerle nombre a la seleccionada",
    en: "give the selected one a name",
  },
  {
    id: "open-out",
    keys: "Ctrl + O",
    mac: "⌘O",
    es: "abrir la seleccionada fuera",
    en: "open the selected one outside",
  },
  {
    id: "unfold",
    keys: "Flecha derecha",
    keysEn: "Right arrow",
    mac: "→",
    es: "abrir o cerrar la tarjeta",
    en: "open or close the card",
  },
  {
    id: "views",
    keys: "Ctrl + 1  ·  Ctrl + 2",
    mac: "⌘1  ·  ⌘2",
    es: "todo  ·  solo lo anclado",
    en: "everything  ·  only what is pinned",
  },
  {
    id: "kinds",
    keys: "Alt + G  ·  Alt + T",
    mac: "⌘G  ·  ⌘T",
    es: "elegir el tipo",
    en: "choose the kind",
  },
  { id: "settings", keys: "F1", mac: "⌘,", es: "abrir Ajustes", en: "open Settings" },
  { id: "dismiss", keys: "Esc", es: "cerrar el panel", en: "close the panel" },
] as const satisfies readonly Row[];

type Which = (typeof PANEL_ROWS)[number]["id"];

let now: Record<keyof Said, string> = ES;

export function adopt(locale: string | null) {
  const asked = locale ?? navigator.language;
  const english = asked.toLowerCase().startsWith("en");
  now = english ? EN : ES;
  document.documentElement.lang = english ? "en" : "es";
}

export function inEnglish() {
  return now === EN;
}

export function t(key: keyof Said) {
  return now[key];
}

export function fill(key: keyof Said, one: string) {
  return now[key].replace("{one}", one);
}

export function items(count: number) {
  return count === 1 ? t("itemOne") : fill("itemMany", String(count));
}

const SPOKEN_ES: Record<string, string> = {
  "⌘": "Comando",
  "⌥": "Opción",
  "⇧": "Mayúsculas",
  "⌃": "Control",
  "⏎": "Enter",
  "⌫": "Retroceso",
  "⇥": "Tabulador",
  "→": "Flecha derecha",
};

const SPOKEN_EN: Record<string, string> = {
  "⌘": "Command",
  "⌥": "Option",
  "⇧": "Shift",
  "⌃": "Control",
  "⏎": "Enter",
  "⌫": "Backspace",
  "⇥": "Tab",
  "→": "Right arrow",
};

// a screen reader says nothing at all for a cell that holds only glyphs
export function spoken(keys: string): string {
  const words = now === EN ? SPOKEN_EN : SPOKEN_ES;
  return [...keys]
    .map((one) => (words[one] ? ` ${words[one]} ` : one))
    .join("")
    .replace(/\s+/g, " ")
    .trim();
}

function spelling(row: Row, mac: boolean): string {
  if (mac && row.mac) {
    return row.mac;
  }
  return now === EN && row.keysEn ? row.keysEn : row.keys;
}

export function panelKeys(mac: boolean): Binding[] {
  return PANEL_ROWS.map((row) => {
    const keys = spelling(row, mac);
    return { id: row.id, keys, said: spoken(keys), does: now === EN ? row.en : row.es };
  });
}

export function panelKey(id: Which, mac: boolean): string {
  const [row] = PANEL_ROWS.filter((one) => one.id === id);
  return spelling(row, mac);
}

// prose needs one way to do it, not the two the table offers
export function oneKey(id: Which, mac: boolean): string {
  return panelKey(id, mac).split("·")[0].trim();
}
