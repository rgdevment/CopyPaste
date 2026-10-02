import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import Welcome from "./ui/Welcome";
import "./index.css";

const root = document.getElementById("root");

if (root) {
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      {window.location.hash === "#welcome" ? <Welcome /> : <App />}
    </React.StrictMode>,
  );
}
