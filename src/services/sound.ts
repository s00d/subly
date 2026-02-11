/**
 * Simple notification sound player using the Web Audio API / HTMLAudioElement.
 * No external dependencies — works with files in /public.
 */

let audioElement: HTMLAudioElement | null = null;

/**
 * Play the notification beep sound.
 * Tries mp3 first, falls back to wav.
 */
export function playNotificationSound(): void {
  try {
    // Reuse existing element to avoid creating many DOM nodes
    if (!audioElement) {
      audioElement = new Audio();
      // Try mp3 first (smaller), wav as fallback
      const canMp3 = audioElement.canPlayType("audio/mpeg");
      audioElement.src = canMp3 ? "/beep.mp3" : "/beep.wav";
    }

    // Reset to start in case it's still playing
    audioElement.currentTime = 0;
    audioElement.volume = 0.7;
    audioElement.play().catch((err) => {
      console.warn("Could not play notification sound:", err);
    });
  } catch (err) {
    console.warn("Notification sound error:", err);
  }
}
