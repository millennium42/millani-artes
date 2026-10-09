import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./ui/App";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Não foi possível iniciar Millani Artes.");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
