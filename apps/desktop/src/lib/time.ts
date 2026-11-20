const pad = (n: number) => String(n).padStart(2, '0');

/** 545 -> « 09h05 » ; au-delà de minuit : « 00h30 (+1 j) ». */
export function hm(minutes: number): string {
  const days = Math.floor(minutes / 1440);
  const m = minutes % 1440;
  const text = `${pad(Math.floor(m / 60))}h${pad(m % 60)}`;
  return days > 0 ? `${text} (+${days} j)` : text;
}

/** 545 -> « 09:05 » (valeur d'un champ <input type="time">). */
export const toInput = (minutes: number) => `${pad(Math.floor(minutes / 60))}:${pad(minutes % 60)}`;

/** « 09:05 » -> 545. */
export function fromInput(value: string): number {
  const [h, m] = value.split(':').map(Number);
  return (h || 0) * 60 + (m || 0);
}

/** « 2026-12-05 » -> « samedi 5 décembre 2026 ». */
export function fmtDate(iso: string): string {
  return new Date(`${iso}T12:00:00`).toLocaleDateString('fr-FR', {
    weekday: 'long', day: 'numeric', month: 'long', year: 'numeric',
  });
}

export const isoDate = (year: number, month0: number, day: number) =>
  `${year}-${pad(month0 + 1)}-${pad(day)}`;

export function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} Ko`;
  return `${(bytes / 1024 / 1024).toFixed(1)} Mo`;
}
