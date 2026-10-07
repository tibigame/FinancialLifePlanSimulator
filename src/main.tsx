import { render } from "preact";
import { installFrontendLogging, reportFrontend } from "./shared/frontendLogging";
import { text } from "./shared/text";
import "./App.css";

installFrontendLogging();
document.title = text("app.name");
void import("./App").then(({ default: App }) => {
  const root = document.getElementById("root");
  if (!root) throw new Error("Root element missing");
  render(<App />, root);
}).catch((error: unknown) => {
  reportFrontend("Error", "app.bootstrap", error);
  const root = document.getElementById("root");
  if (root) root.textContent = text("app.renderError");
});
