// Lucide-style inline SVG icons for MoAI Studio
// All use stroke=currentColor; size via width/height props.

const Ic = ({ d, size = 14, strokeWidth = 1.75, ...rest }) => (
  <svg viewBox="0 0 24 24" width={size} height={size} className="ic"
       strokeWidth={strokeWidth} stroke="currentColor" fill="none"
       strokeLinecap="round" strokeLinejoin="round" {...rest}>
    {d}
  </svg>
);

const I = {
  folder:    (p) => <Ic {...p} d={<><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></>}/>,
  folderOpen:(p) => <Ic {...p} d={<><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2"/><path d="M3 9h18l-2 9a2 2 0 0 1-2 1.5H5a2 2 0 0 1-2-1.5z"/></>}/>,
  file:      (p) => <Ic {...p} d={<><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/></>}/>,
  fileCode:  (p) => <Ic {...p} d={<><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/><path d="M10 13l-2 2 2 2"/><path d="M14 13l2 2-2 2"/></>}/>,
  fileText:  (p) => <Ic {...p} d={<><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/><path d="M9 13h6M9 17h4"/></>}/>,
  chev:      (p) => <Ic {...p} d={<path d="M9 6l6 6-6 6"/>}/>,
  search:    (p) => <Ic {...p} d={<><circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/></>}/>,
  files:     (p) => <Ic {...p} d={<><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/></>}/>,
  git:       (p) => <Ic {...p} d={<><circle cx="6" cy="6" r="2.4"/><circle cx="18" cy="6" r="2.4"/><circle cx="12" cy="18" r="2.4"/><path d="M6 8.4v3.6a3 3 0 0 0 3 3h2.5"/><path d="M18 8.4V12"/></>}/>,
  spec:      (p) => <Ic {...p} d={<><rect x="4" y="4" width="6" height="6" rx="1.4"/><rect x="14" y="4" width="6" height="6" rx="1.4"/><rect x="4" y="14" width="6" height="6" rx="1.4"/><rect x="14" y="14" width="6" height="6" rx="1.4"/></>}/>,
  agent:     (p) => <Ic {...p} d={<><circle cx="12" cy="12" r="3"/><path d="M12 5v-2M12 21v-2M5 12h-2M21 12h-2M7.05 7.05L5.6 5.6M18.4 18.4l-1.45-1.45M16.95 7.05L18.4 5.6M5.6 18.4l1.45-1.45"/></>}/>,
  terminal:  (p) => <Ic {...p} d={<><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 9l3 3-3 3M13 15h4"/></>}/>,
  globe:     (p) => <Ic {...p} d={<><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/></>}/>,
  settings:  (p) => <Ic {...p} d={<><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1A2 2 0 1 1 4.3 17l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8L4.2 7a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/></>}/>,
  plus:      (p) => <Ic {...p} d={<path d="M12 5v14M5 12h14"/>}/>,
  x:         (p) => <Ic {...p} d={<path d="M6 6l12 12M18 6L6 18"/>}/>,
  check:     (p) => <Ic {...p} d={<path d="M5 12l5 5L20 7"/>}/>,
  alertTri:  (p) => <Ic {...p} d={<><path d="M12 3l10 18H2z"/><path d="M12 9v5M12 18h.01"/></>}/>,
  zap:       (p) => <Ic {...p} d={<path d="M13 2L4 14h7l-1 8 9-12h-7z"/>}/>,
  branch:    (p) => <Ic {...p} d={<><circle cx="6" cy="6" r="2.4"/><circle cx="6" cy="18" r="2.4"/><circle cx="18" cy="6" r="2.4"/><path d="M6 8.4v7.2"/><path d="M18 8.4a6 6 0 0 1-6 6"/></>}/>,
  play:      (p) => <Ic {...p} d={<path d="M6 4l14 8-14 8z"/>}/>,
  pause:     (p) => <Ic {...p} d={<><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></>}/>,
  stop:      (p) => <Ic {...p} d={<rect x="5" y="5" width="14" height="14" rx="1.5"/>}/>,
  refresh:   (p) => <Ic {...p} d={<><path d="M21 12a9 9 0 1 1-3-6.7"/><path d="M21 4v5h-5"/></>}/>,
  wrench:    (p) => <Ic {...p} d={<path d="M14.7 6.3a4 4 0 0 0 5 5l-9.5 9.5a2 2 0 0 1-2.8 0L5.2 18.6a2 2 0 0 1 0-2.8z"/>}/>,
  msg:       (p) => <Ic {...p} d={<path d="M21 11.5a8.4 8.4 0 0 1-9 8.5 8.5 8.5 0 0 1-3.4-.7L3 21l1.7-5.6A8.5 8.5 0 1 1 21 11.5z"/>}/>,
  cpu:       (p) => <Ic {...p} d={<><rect x="6" y="6" width="12" height="12" rx="2"/><rect x="9" y="9" width="6" height="6"/><path d="M9 1v3M15 1v3M9 20v3M15 20v3M1 9h3M1 15h3M20 9h3M20 15h3"/></>}/>,
  dollar:    (p) => <Ic {...p} d={<path d="M12 2v20M17 6H9.5a3.5 3.5 0 1 0 0 7h5a3.5 3.5 0 1 1 0 7H6"/>}/>,
  inbox:     (p) => <Ic {...p} d={<><path d="M22 12h-6l-2 3h-4l-2-3H2"/><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/></>}/>,
  panelL:    (p) => <Ic {...p} d={<><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/></>}/>,
  panelR:    (p) => <Ic {...p} d={<><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M15 4v16"/></>}/>,
  splitH:    (p) => <Ic {...p} d={<><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 12h18"/></>}/>,
  splitV:    (p) => <Ic {...p} d={<><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M12 4v16"/></>}/>,
  arrowDown: (p) => <Ic {...p} d={<path d="M12 5v14M19 12l-7 7-7-7"/>}/>,
};

window.I = I;
