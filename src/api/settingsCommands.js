import { invoke } from "@tauri-apps/api/core";

export async function checkForUpdatesCommand() {
  return await invoke("check_for_updates");
}
