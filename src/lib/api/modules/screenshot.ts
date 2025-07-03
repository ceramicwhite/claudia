import { invoke } from "@tauri-apps/api/core";

/**
 * Screenshot API module
 */
export const screenshotApi = {
  /**
   * Captures a screenshot of a specific region in the window
   * @param url - The URL to capture
   * @param selector - Optional selector to capture
   * @param fullPage - Whether to capture the full page
   * @returns Promise resolving to the path of the saved screenshot
   */
  async captureUrlScreenshot(
    url: string,
    selector?: string | null,
    fullPage: boolean = false
  ): Promise<string> {
    return await invoke<string>("capture_url_screenshot", {
      url,
      selector,
      fullPage,
    });
  },

  /**
   * Cleans up old screenshot files from the temporary directory
   * @param olderThanMinutes - Remove files older than this many minutes (default: 60)
   * @returns Promise resolving to the number of files deleted
   */
  async cleanupScreenshotTempFiles(olderThanMinutes?: number): Promise<number> {
    try {
      return await invoke<number>("cleanup_screenshot_temp_files", { olderThanMinutes });
    } catch (error) {
      console.error("Failed to cleanup screenshot files:", error);
      throw error;
    }
  },
};