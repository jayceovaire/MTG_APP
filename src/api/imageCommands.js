import { invoke } from "@tauri-apps/api/core";

export async function getMostRecentCachedImageCommand() {
  return await invoke("get_most_recent_cached_image");
}

export async function getBase64ImagesCommand(paths) {
  return await invoke("get_base64_images", { paths });
}

export async function fetchCardImagesCommand() {
  return await invoke("fetch_card_images");
}
