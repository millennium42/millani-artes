import { renderToStaticMarkup } from "react-dom/server";
import { expect, test } from "vitest";
import App from "./App";

test("apresenta a marca no heading do conteúdo principal", () => {
  const markup = renderToStaticMarkup(<App />);

  expect(markup).toMatch(/^<main>.*<h1>Millani Artes<\/h1>.*<\/main>$/);
});
