/** Egyptian notes and coins in piastres, mirroring pos-core `cash::closing::DENOMINATIONS`
 * (200 EGP … 25 piastres); the backend refuses any other (`count_invalid`). */
export const DENOMINATIONS = [20_000, 10_000, 5_000, 2_000, 1_000, 500, 100, 50, 25] as const;

/** 5 EGP and above are notes in the count sheet labels. */
export function isNote(denomination: number): boolean {
  return denomination >= 500;
}

export type Counts = Record<number, string>;

export function emptyCounts(): Counts {
  return Object.fromEntries(DENOMINATIONS.map((d) => [d, ""]));
}

/** Pieces counted per denomination, as integers (empty = 0). */
export function countLines(counts: Counts): { denomination: number; count: number }[] {
  return DENOMINATIONS.map((denomination) => ({
    denomination,
    count: Number(counts[denomination] || "0"),
  })).filter((l) => l.count > 0);
}

/** The total the sheet shows while typing (integer centimes; the backend recomputes it). */
export function countedTotal(counts: Counts): number {
  return countLines(counts).reduce((sum, l) => sum + l.denomination * l.count, 0);
}
