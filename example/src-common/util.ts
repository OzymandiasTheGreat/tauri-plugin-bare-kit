export function randint(min = 1, max = Number.MAX_SAFE_INTEGER): number {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min + 1)) + min
}
