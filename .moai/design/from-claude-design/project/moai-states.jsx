// MoAI Studio — state mocks (Empty, Loading, Error, First-run, Agent running header)

const { I } = window;

function EmptyState({ kind = "no-workspace" }) {
  if (kind === "no-workspace") {
    return (
      <div className="empty">
        <img src="assets/moai-logo-3.png" alt="" />
        <h3>Open a workspace</h3>
        <p>Pick a git repository to start. MoAI Studio will load <code>.moai/specs</code> and restore your last pane layout.</p>
        <button className="pri-btn">Open folder…</button>
        <div className="hint" style={{ marginTop: 14 }}>or press <kbd>⌘O</kbd></div>
      </div>
    );
  }
  if (kind === "no-file") {
    return (
      <div className="empty">
        <I.fileText size={56} />
        <h3 style={{ marginTop: 18 }}>No file selected</h3>
        <p>Pick a file from the explorer, or search across the workspace.</p>
        <div className="hint">Search files <kbd>⌘P</kbd> · Open recent <kbd>⌘E</kbd></div>
      </div>
    );
  }
  return (
    <div className="empty">
      <I.inbox size={56} />
      <h3 style={{ marginTop: 18 }}>Nothing here yet</h3>
      <p>Create your first SPEC to begin tracking acceptance criteria.</p>
      <button className="pri-btn">New SPEC…</button>
    </div>
  );
}

function LoadingSkeleton() {
  return (
    <div style={{ padding: "8px 0" }}>
      {Array.from({ length: 9 }).map((_, i) => (
        <div key={i} className="skel-row" style={{ display:"grid", gridTemplateColumns: "16px 1fr 24px", gap: 10, alignItems: "center" }}>
          <div className="skel" style={{ width: 14, height: 14, borderRadius: 3 }}/>
          <div className="skel" style={{ width: `${50 + (i*7)%40}%` }}/>
          <div className="skel" style={{ width: 14, height: 8 }}/>
        </div>
      ))}
    </div>
  );
}

function ErrorBanner({ msg = "Failed to read src/panes/persist.rs · permission denied", code = "EACCES" }) {
  return (
    <div className="errbar">
      <I.alertTri size={16}/>
      <div>
        <div><strong>Cannot open file</strong> · {msg}</div>
        <div style={{ fontSize: 11, color: "var(--fg-3)", fontFamily: "var(--font-mono)", marginTop: 2 }}>code: {code} · 13:48:02</div>
      </div>
      <div className="actions">
        <button>Dismiss</button>
        <button className="pri">Retry · ⌘R</button>
      </div>
    </div>
  );
}

function FirstRun() {
  return (
    <div className="firstrun">
      <div className="panel">
        <img className="mascot" src="assets/moai-logo-3.png" alt="MoAI mascot"/>
        <h2>Welcome to MoAI Studio</h2>
        <p className="lede">An agentic IDE for the moai-adk workflow — terminal, code, and Claude progress in one canvas. Three quick steps to get going.</p>
        <div className="steps">
          <div className="step done">
            <span className="n"><I.check size={12}/></span>
            <div><div className="t">Pick a workspace</div><div className="d">~/code/moai-studio</div></div>
            <span className="x">⌘O</span>
          </div>
          <div className="step active">
            <span className="n">2</span>
            <div><div className="t">Load .moai/specs</div><div className="d">Found 11 specs · 3 in dev</div></div>
            <span className="x">scanning…</span>
          </div>
          <div className="step">
            <span className="n">3</span>
            <div><div className="t">Restore last layout</div><div className="d">Terminal + Code + Markdown · 4 tabs</div></div>
            <span className="x">⌘1</span>
          </div>
        </div>
        <div className="actions">
          <button className="sec">Skip tour</button>
          <button className="pri">Continue → workspace</button>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { EmptyState, LoadingSkeleton, ErrorBanner, FirstRun });
