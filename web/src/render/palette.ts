import { Color3 } from "@babylonjs/core";

/**
 * Paleta única do jogo. Toda cor usada em modelos e cenário sai daqui, para
 * manter o visual coeso (ver docs/ART_STYLE.md).
 */
const hex = (h: string) => Color3.FromHexString(h);

export const PALETTE = {
  // Pinturas de cabine
  paintRed: hex("#d23a2c"),
  paintOrange: hex("#e8902c"),
  paintYellow: hex("#f0c02e"),
  paintBlue: hex("#2f6fd0"),
  paintGreen: hex("#93ad5c"),
  paintWhite: hex("#e9edf0"),

  // Veículo
  chassis: hex("#5d626b"),
  chassisDark: hex("#474b52"),
  tire: hex("#2c2d31"),
  rim: hex("#e4e7ea"),
  hub: hex("#55585e"),
  grille: hex("#1d1f23"),
  trim: hex("#b9bec5"),
  glass: hex("#cfe0ea"),
  headlight: hex("#fff1c8"),
  taillight: hex("#ff3a2c"),
  indicator: hex("#ff9a1c"),
  stripeRed: hex("#d9392b"),
  stripeWhite: hex("#f3f3f0"),

  // Cenário
  grassA: hex("#cdbb62"),
  grassB: hex("#c4b65a"),
  grassC: hex("#b9b65e"),
  grassD: hex("#d6c46c"),
  hillLow: hex("#a9b357"),
  hillHigh: hex("#d9c879"),
  asphalt: hex("#4b4e56"),
  roadLine: hex("#f2f0e4"),
  leafA: hex("#c8d84a"),
  leafB: hex("#a6c64c"),
  leafC: hex("#e0c548"),
  trunk: hex("#7b5a3d"),
  rock: hex("#d8dbde"),
  cloud: hex("#ffffff"),
  ramp: hex("#b99c70"),
  cone: hex("#f07a22"),
  crate: hex("#c08a52"),

  // Céu e névoa
  skyTop: hex("#7fb0dc"),
  skyHorizon: hex("#e3ecf0"),
  sun: hex("#fff3d6"),
} as const;

export type PaletteKey = keyof typeof PALETTE;

/** Pinturas disponíveis para a cabine, em ordem de troca (tecla P). */
export const PAINTS: ReadonlyArray<{ name: string; color: Color3 }> = [
  { name: "vermelho", color: PALETTE.paintRed },
  { name: "laranja", color: PALETTE.paintOrange },
  { name: "amarelo", color: PALETTE.paintYellow },
  { name: "azul", color: PALETTE.paintBlue },
  { name: "verde", color: PALETTE.paintGreen },
  { name: "branco", color: PALETTE.paintWhite },
];
