import { invoke } from "@tauri-apps/api/core";

export async function checkForUpdatesCommand() {
  return await invoke("check_for_updates");
}

export async function installUpdateCommand() {
  return await invoke("install_update");
}
