import { openUrl } from "@tauri-apps/plugin-opener";

window.addEventListener("DOMContentLoaded", () => {
  console.log("Wingman initialized");

  const launchButton = document.querySelector(
    "#launch-button",
  ) as HTMLButtonElement | null;

  launchButton?.addEventListener("click", async () => {
    try {
      await openUrl("https://copilot.microsoft.com");
      console.log("Copilot opened");
    } catch (error) {
      console.error("Failed to open Copilot:", error);
    }
  });
});