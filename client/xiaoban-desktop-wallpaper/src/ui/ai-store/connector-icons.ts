const svg=(p: string)=>`<svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">${p}</svg>`;
const marks: Record<string, string> = {
github:svg('<circle cx="18" cy="18" r="13" fill="#1d2026"/><path d="M12 28v-4q-4-1-5-5m17 9v-4q4-1 5-5" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round"/><path d="M10 14Q9 9 13 9l2 2q3-1 6 0l2-2q4 0 3 5 3 4-1 8-2 2-7 2s-5 0-7-2q-4-4-1-8" fill="white"/>'),
feishu:svg('<path d="M3 17 28 6l-7 22L3 17z" fill="#3177e3"/><path d="M11 21 28 6l5 12-14 12z" fill="#17b8be"/><path d="M3 17 18 18l-7 3z" fill="#7bc2f4"/>'),
notion:svg('<rect x="5" y="5" width="26" height="26" fill="white" stroke="#1d1d1d" stroke-width="2.8"/><path d="m10 10 5-1 11 2-5 2-11-1z" fill="#1d1d1d"/><text x="10" y="27" fill="#151515" font-family="Georgia" font-size="21" font-weight="bold">N</text>'),
drive:svg('<path d="M13 3h10l11 20H24z" fill="#ffba00"/><path d="M13 3h10L11 25H1z" fill="#22a766"/><path d="M11 25h23l-5 8H6z" fill="#4284ed"/>'),
figma:svg('<rect x="11" y="3" width="10" height="10" rx="5" fill="#f24e1e"/><rect x="11" y="13" width="10" height="10" rx="5" fill="#a259ff"/><rect x="11" y="23" width="10" height="10" rx="5" fill="#0acf83"/><rect x="21" y="3" width="10" height="10" rx="5" fill="#ff7262"/><circle cx="26" cy="18" r="5" fill="#1abcfe"/>'),
slack:svg('<rect x="12" y="1" width="7" height="15" rx="3.5" fill="#36c5f0"/><rect x="20" y="12" width="15" height="7" rx="3.5" fill="#2eb67d"/><rect x="17" y="20" width="7" height="15" rx="3.5" fill="#ecb22e"/><rect x="1" y="17" width="15" height="7" rx="3.5" fill="#e01e5a"/>'),
gmail:svg('<path d="M4 27V9l14 11L32 9v18" fill="none" stroke="#ea4335" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/><path d="M4 10v18" stroke="#4285f4" stroke-width="5"/><path d="M32 10v18" stroke="#34a853" stroke-width="5"/>'),
linear:svg('<circle cx="18" cy="18" r="15" fill="#5e6ad2"/><path d="M6 12 24 30M9 8l19 19M13 5l18 18" stroke="#fff" stroke-width="2.4" stroke-linecap="round"/>')};
export function connectorIcon(id: string) { return "data:image/svg+xml," + encodeURIComponent(marks[id] ?? ""); }

