import { getIconCollections, iconsPlugin } from "@egoist/tailwindcss-icons";
import type { Config } from "tailwindcss";

export default {
  plugins: [
    iconsPlugin({
      collections: getIconCollections(["mdi"]),
      scale: 1.15,
    }),
  ],
} satisfies Config;
