import { stopClientProcess } from "./global-setup-client";

export default async function globalTeardown() {
  await stopClientProcess();
}
