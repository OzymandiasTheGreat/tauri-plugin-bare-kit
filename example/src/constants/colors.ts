const GreyDark = "#212121"
const GreyDarker = "#424242"
const GreyLight = "#fafafa"
const GreyLighter = "#eeeeee"
const ErrorRed = "#e53935"

export const Bubbles = [
  "#ec407a", // pink lighten-1
  "#ab47bc", // purple lighten-1
  "#3949ab", // indigo darken-1
  "#03a9f4", // light-blue
  "#00897b", // teal darken-1
  "#8bc34a", // light-green
  "#ffeb3b", // yellow
  "#ffb300", // amber darken-1
]

export default {
  light: {
    text: GreyDark,
    background: GreyLight,
    error: ErrorRed,
    input: GreyLighter,
    self: GreyLighter,
  },
  dark: {
    text: GreyLight,
    background: GreyDark,
    error: ErrorRed,
    input: GreyDarker,
    self: GreyDarker,
  },
}
