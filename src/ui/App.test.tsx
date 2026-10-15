import { renderToStaticMarkup } from "react-dom/server";
import { expect, test } from "vitest";
import App from "./App";

test("apresenta a marca no heading do conteúdo principal", () => {
  const markup = renderToStaticMarkup(<App />);

  expect(markup.match(/<header>/g)).toHaveLength(1);
  expect(markup).toMatch(
    /^<header>.*<\/header><main>.*<h1>Millani Artes<\/h1>.*<\/main>$/,
  );
  expect(markup).toContain("Aplicativo local em construção.");
});
