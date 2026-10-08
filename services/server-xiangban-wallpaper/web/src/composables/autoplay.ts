/** Only one muted preview plays at a time; starting another pauses the previous (saves battery on phones). */
let current: HTMLVideoElement | null = null

export function playExclusive(video: HTMLVideoElement) {
  if (current && current !== video) current.pause()
  current = video
  video.play().catch(() => {
    // autoplay can be blocked until the user interacts; the poster image stays visible
  })
}

export function pauseVideo(video: HTMLVideoElement) {
  video.pause()
  if (current === video) current = null
}
