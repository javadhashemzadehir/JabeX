import localFont from "next/font/local";

/** Persian UI face. FaNum cuts draw Latin digits as Persian numerals. */
export const yekanBakh = localFont({
  src: [
    {
      path: "../fonts/Yekan-Bakh-FaNum-04-Regular.woff",
      weight: "400",
      style: "normal",
    },
    {
      path: "../fonts/Yekan-Bakh-FaNum-05-Medium.woff",
      weight: "500",
      style: "normal",
    },
    {
      path: "../fonts/Yekan-Bakh-FaNum-06-Bold.woff",
      weight: "700",
      style: "normal",
    },
  ],
  variable: "--font-yekan",
  display: "swap",
  adjustFontFallback: false,
});

/** English UI face. Persian glyphs fall back to Yekan Bakh. */
export const comfortaa = localFont({
  src: [
    {
      path: "../fonts/Comfortaa-Regular.ttf",
      weight: "400",
      style: "normal",
    },
    {
      path: "../fonts/Comfortaa-Medium.ttf",
      weight: "500",
      style: "normal",
    },
    {
      path: "../fonts/Comfortaa-Bold.ttf",
      weight: "700",
      style: "normal",
    },
  ],
  variable: "--font-comfortaa",
  display: "swap",
  adjustFontFallback: false,
});
