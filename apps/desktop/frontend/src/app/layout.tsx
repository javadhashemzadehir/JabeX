import type { Metadata } from "next";
import { AppProviders } from "@/components/providers/app-providers";
import { DEFAULT_LOCALE, getDirection } from "@/lib/i18n/config";
import { comfortaa, yekanBakh } from "@/lib/fonts";
import "@/styles/globals.css";

export const metadata: Metadata = {
  title: "JabeX",
  description: "Private, offline-first digital Leitner Box",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const direction = getDirection(DEFAULT_LOCALE);

  return (
    <html
      lang={DEFAULT_LOCALE}
      dir={direction}
      className={`${yekanBakh.variable} ${comfortaa.variable}`}
      suppressHydrationWarning
    >
      <body className="min-h-svh font-sans antialiased">
        <AppProviders>{children}</AppProviders>
      </body>
    </html>
  );
}
