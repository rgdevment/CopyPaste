import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { onMac, reachable } from "./core";
import Ready from "./ui/Ready";
import Welcome from "./ui/Welcome";
import "./index.css";

const PAGES: Record<string, () => React.JSX.Element | null> = {
  "#welcome": Welcome,
  "#ready": Ready,
};
const Page = PAGES[window.location.hash] ?? App;

const root = document.getElementById("root");

if (root) {
  if (onMac()) {
    // WebKit on a Mac skips buttons and lists on Tab unless each one asks for it
    new MutationObserver(() => reachable(root)).observe(root, { childList: true, subtree: true });
  }
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <Page />
    </React.StrictMode>,
  );
}
