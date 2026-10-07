import type { ComponentChildren } from "preact";
import { useErrorBoundary } from "preact/hooks";
import { reportFrontend } from "../shared/frontendLogging";
import { text } from "../shared/text";

export function PageBoundary({ children }: { children: ComponentChildren }) {
  const [error, reset] = useErrorBoundary((reason) => reportFrontend("Error", "preact.render", reason));
  return error ? <div role="alert" style={{ padding: "32px" }}>
    <p>{text("app.renderError")}</p>
    <button onClick={reset}>{text("app.retry")}</button>
  </div> : <>{children}</>;
}
