// MoAI Studio — main app (Tweaks panel + Design Canvas composing surfaces)

const { I, FileTree, SAMPLE_TREE, Terminal, CodeViewer, MarkdownViewer,
        AgentDashboard, GitMgmt, SpecKanban, WebBrowser,
        EmptyState, LoadingSkeleton, ErrorBanner, FirstRun,
        useTweaks, TweaksPanel, TweakSection, TweakSlider, TweakToggle,
        TweakRadio, TweakSelect, TweakColor,
        DesignCanvas, DCSection, DCArtboard,
        CmdPalette, CommandPalette, SlashBar, FindReplace, CodeStub,
        LspHover, MXPopover, MergeDiff, SprintPanel, SettingsModal,
        CrashBanner, UpdateBanner, LspStarting, PtyStarting, WorkspaceSwitching } = window;

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "dark",
  "density": "comfortable",
  "accent": "teal",
  "sidebarSide": "left",
  "paneLayout": "4",
  "agentSlot": "right",
  "showSurface": "all"
}/*EDITMODE-END*/;

const ACCENTS = {
  teal:   { c: "#144a46", soft: "rgba(20,74,70,0.14)" },
  blue:   { c: "#2563EB", soft: "rgba(37,99,235,0.14)" },
  violet: { c: "#6a4cc7", soft: "rgba(106,76,199,0.16)" },
  cyan:   { c: "#06B6D4", soft: "rgba(6,182,212,0.16)" },
};

// ── App shell wrapper ─────────────────────────────────────────────────
function MoAI({ mode = "dark", density = "comfortable", children, accent, sidebar, agentSlot = "right", paneLayout = "4", title = "moai-studio", showStatus = true, showAgent = true }) {
  const acc = ACCENTS[accent] || ACCENTS.teal;
  const cssVars = { "--accent": acc.c, "--accent-soft": acc.soft };
  if (density === "compact") cssVars["--row-h"] = "22px";
  return (
    <div className="moai" data-mode={mode} style={cssVars}>
      <div className="moai-top">
        <div className="moai-traffic"><span/><span/><span/></div>
        <div className="moai-cmdbar">
          <I.search size={12}/>
          <span style={{ color: "var(--fg-2)", fontSize: 11 }}>{title}</span>
          <span style={{ color: "var(--fg-3)" }}>›</span>
          <span>SPEC-V3-006 · render.rs</span>
          <span className="kb">⌘P</span>
        </div>
        <div className="moai-top-right">
          {showAgent && <span className="moai-agent-pill"><span className="moai-agent-dot"/> Agent · MS-1 · 3:42</span>}
          <span style={{ display:"flex", gap: 4, color: "var(--fg-3)" }}>
            <I.panelL size={14}/><I.splitH size={14}/><I.splitV size={14}/>
          </span>
          <span style={{ color: "var(--fg-3)" }}><I.settings size={14}/></span>
        </div>
      </div>
      <div className="moai-main">
        {sidebar !== "none" && sidebar !== "right" && <Sidebar density={density} />}
        <div className="moai-content">
          {children}
        </div>
        {sidebar === "right" && <Sidebar density={density} />}
      </div>
      {showStatus && (
        <div className="moai-status">
          <span><I.branch size={11} style={{verticalAlign:-1}}/> feature/v3-006-md</span>
          <span className="accent">↑3 ↓1</span>
          <span>● 4 modified · 2 staged</span>
          <span className="right">
            <span>UTF-8</span><span>LF</span><span>Rust</span>
            <span style={{ color: "var(--mint)" }}>● rust-analyzer</span>
            <span>$1.23 today</span>
          </span>
        </div>
      )}
    </div>
  );
}

function Sidebar({ density }) {
  return (
    <div className="moai-side">
      <div className="moai-brand">
        <img className="mascot" src="assets/moai-logo-3.png" alt=""/>
        <span className="name">MoAI Studio</span>
        <span className="ws">v0.3</span>
      </div>
      <div className="moai-nav">
        <div className="moai-nav-item active"><I.files size={14}/>Files<span className="badge">142</span></div>
        <div className="moai-nav-item"><I.search size={14}/>Search</div>
        <div className="moai-nav-item"><I.git size={14}/>Git<span className="badge">4</span></div>
        <div className="moai-nav-item"><I.spec size={14}/>SPEC<span className="badge">11</span></div>
        <div className="moai-nav-item"><I.agent size={14}/>Agent<span className="badge" style={{ color: "var(--mint)" }}>●</span></div>
      </div>
      <div className="moai-side-section">Workspace<span className="count">moai-studio</span></div>
      <FileTree items={SAMPLE_TREE} density={density}/>
    </div>
  );
}

// ── Surface w/ tab strip wrapper ──────────────────────────────────────
function Surface({ tabs = [], children }) {
  return (
    <>
      <div className="moai-tabs">
        {tabs.map((t, i) => (
          <div key={i} className={"moai-tab " + (t.active ? "active" : "")}>
            {t.dirty && <span className="dot"/>}
            {!t.dirty && (t.icon || <I.file size={12}/>)}
            <span>{t.name}</span>
            <I.x className="x" size={10}/>
          </div>
        ))}
        <div className="moai-tab-spacer"/>
        <div className="moai-tabs-actions"><I.plus size={13}/></div>
      </div>
      <div className="moai-canvas">
        {children}
      </div>
    </>
  );
}

// ── Specific layouts ──────────────────────────────────────────────────
function FullLayout({ mode, accent, density, agentSlot = "right", sidebarSide = "left", paneLayout = "4" }) {
  const tabs = [
    { name: "render.rs", active: true, dirty: true, icon: <I.fileCode size={12}/> },
    { name: "SPEC-V3-006.md", icon: <I.fileText size={12}/> },
    { name: "tree.rs", dirty: true, icon: <I.fileCode size={12}/> },
    { name: "Terminal", icon: <I.terminal size={12}/> },
    { name: "Agent", icon: <I.agent size={12}/> },
  ];

  // Pane layout matrix
  let panes;
  if (paneLayout === "1") {
    panes = (<div className="moai-panes" style={{ gridTemplateColumns: "1fr" }}>
      <div className="moai-pane"><div className="moai-pane-head"><I.fileCode size={12}/>render.rs</div><div className="moai-pane-body"><CodeViewer/></div></div>
    </div>);
  } else if (paneLayout === "2") {
    panes = (<div className="moai-panes" style={{ gridTemplateColumns: "1fr 1fr" }}>
      <div className="moai-pane"><div className="moai-pane-head"><I.fileCode size={12}/>render.rs</div><div className="moai-pane-body"><CodeViewer/></div></div>
      <div className="moai-pane"><div className="moai-pane-head"><I.fileText size={12}/>SPEC-V3-006.md</div><div className="moai-pane-body"><MarkdownViewer/></div></div>
    </div>);
  } else if (paneLayout === "3") {
    panes = (<div className="moai-panes" style={{ gridTemplateColumns: "1.4fr 1fr", gridTemplateRows: "1.4fr 1fr" }}>
      <div className="moai-pane" style={{ gridRow: "1 / 3" }}><div className="moai-pane-head"><I.fileCode size={12}/>render.rs</div><div className="moai-pane-body"><CodeViewer/></div></div>
      <div className="moai-pane"><div className="moai-pane-head"><I.fileText size={12}/>SPEC-V3-006.md</div><div className="moai-pane-body"><MarkdownViewer/></div></div>
      <div className="moai-pane"><div className="moai-pane-head"><I.terminal size={12}/>terminal · zsh</div><div className="moai-pane-body"><Terminal/></div></div>
    </div>);
  } else {
    // 4-pane (default)
    panes = (<div className="moai-panes" style={{ gridTemplateColumns: "1.4fr 1fr", gridTemplateRows: "1fr 1fr" }}>
      <div className="moai-pane"><div className="moai-pane-head"><I.fileCode size={12}/>src/panes/tree.rs</div><div className="moai-pane-body"><CodeViewer/></div></div>
      <div className="moai-pane"><div className="moai-pane-head"><I.fileText size={12}/>SPEC-V3-006.md</div><div className="moai-pane-body"><MarkdownViewer/></div></div>
      <div className="moai-pane"><div className="moai-pane-head"><I.terminal size={12}/>terminal · zsh</div><div className="moai-pane-body"><Terminal/></div></div>
      <div className="moai-pane"><div className="moai-pane-head"><I.agent size={12}/>Agent · live timeline</div><div className="moai-pane-body" style={{ overflow:"hidden" }}><AgentDashboard/></div></div>
    </div>);
  }

  return (
    <MoAI mode={mode} accent={accent} density={density} sidebar={sidebarSide}>
      <Surface tabs={tabs}>{panes}</Surface>
    </MoAI>
  );
}

function SurfaceFrame({ title, mode, accent, density, w = 880, h = 560, surface, withSidebar = false, sidebar="left" }) {
  const tabs = [{ name: title, active: true, icon: <I.fileCode size={12}/> }];
  return (
    <MoAI mode={mode} accent={accent} density={density} sidebar={withSidebar ? sidebar : "none"} showStatus={false} showAgent={false}>
      <Surface tabs={tabs}>
        <div style={{ flex:1, display:"flex", minWidth:0 }}>{surface}</div>
      </Surface>
    </MoAI>
  );
}

// ── App / Tweaks ──────────────────────────────────────────────────────
function App() {
  const [tw, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const mode = tw.theme;
  const accent = tw.accent;
  const density = tw.density;
  const agentSlot = tw.agentSlot;
  const paneLayout = tw.paneLayout;
  const sidebarSide = tw.sidebarSide;

  return (<>
    <TweaksPanel title="Tweaks">
      <TweakSection label="Theme"/>
      <TweakRadio label="Mode" value={tw.theme} options={["dark","light"]} onChange={(v)=>setTweak("theme",v)}/>
      <TweakRadio label="Accent" value={tw.accent} options={["teal","blue","violet","cyan"]} onChange={(v)=>setTweak("accent",v)}/>
      <TweakSection label="Layout"/>
      <TweakRadio label="Density" value={tw.density} options={["compact","comfortable"]} onChange={(v)=>setTweak("density",v)}/>
      <TweakRadio label="Sidebar" value={tw.sidebarSide} options={["left","right"]} onChange={(v)=>setTweak("sidebarSide",v)}/>
      <TweakRadio label="Panes" value={tw.paneLayout} options={["1","2","3","4"]} onChange={(v)=>setTweak("paneLayout",v)}/>
      <TweakSection label="Agent"/>
      <TweakRadio label="Slot" value={tw.agentSlot} options={["right","bottom","tab"]} onChange={(v)=>setTweak("agentSlot",v)}/>
    </TweaksPanel>
    <DesignCanvas>
      {/* === Section 1: Main layout === */}
      <DCSection id="main" title="Main Layout — Workspace" subtitle="Sidebar · Tabs · 4-pane · Agent · Status. Try the Tweaks panel.">
        <DCArtboard id="main-dark" label={`Dark · ${paneLayout}-pane · ${density}`} width={1480} height={920}>
          <FullLayout mode="dark" accent={accent} density={density} agentSlot={agentSlot} sidebarSide={sidebarSide} paneLayout={paneLayout}/>
        </DCArtboard>
        <DCArtboard id="main-light" label={`Light · ${paneLayout}-pane · ${density}`} width={1480} height={920}>
          <FullLayout mode="light" accent={accent} density={density} agentSlot={agentSlot} sidebarSide={sidebarSide} paneLayout={paneLayout}/>
        </DCArtboard>
      </DCSection>

      {/* === Section 2: Terminal & Panes (Tier 0) === */}
      <DCSection id="t0" title="Tier 0 · Terminal + Panes/Tabs" subtitle="Locked surfaces — design alignment with shipped impl.">
        <DCArtboard id="t-dark" label="Terminal · dark" width={780} height={520}>
          <SurfaceFrame title="terminal · zsh" mode="dark" accent={accent} density={density}
            surface={<Terminal/>} />
        </DCArtboard>
        <DCArtboard id="t-light" label="Terminal · light" width={780} height={520}>
          <SurfaceFrame title="terminal · zsh" mode="light" accent={accent} density={density}
            surface={<Terminal/>} />
        </DCArtboard>
        <DCArtboard id="t-tabs" label="Pane split · 3-up demo" width={1100} height={560}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[
              { name: "render.rs", active: true, icon: <I.fileCode size={12}/> },
              { name: "SPEC-V3-006.md", icon: <I.fileText size={12}/> },
              { name: "Terminal", icon: <I.terminal size={12}/> },
            ]}>
              <div className="moai-panes" style={{ gridTemplateColumns: "1fr 1fr", gridTemplateRows: "1fr 1fr" }}>
                <div className="moai-pane" style={{ gridRow: "1/3" }}>
                  <div className="moai-pane-head"><I.fileCode size={12}/>render.rs · focused</div>
                  <div className="moai-pane-body"><CodeViewer/></div>
                </div>
                <div className="moai-pane">
                  <div className="moai-pane-head"><I.fileText size={12}/>SPEC-V3-006.md</div>
                  <div className="moai-pane-body"><MarkdownViewer/></div>
                </div>
                <div className="moai-pane">
                  <div className="moai-pane-head"><I.terminal size={12}/>terminal</div>
                  <div className="moai-pane-body"><Terminal/></div>
                </div>
              </div>
            </Surface>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 3: File Explorer === */}
      <DCSection id="fe" title="File Explorer (V3-005)" subtitle="Git status badges · fuzzy search · empty + loading states">
        <DCArtboard id="fe-pop" label="Populated · dark" width={300} height={580}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="left" showStatus={false} showAgent={false}>
            <div style={{ display:"none" }}/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="fe-search" label="Cmd+P · fuzzy search" width={420} height={420}>
          <div style={{ background: "#0e1513", height: "100%", padding: 32, display:"flex", flexDirection:"column", gap: 0, fontFamily:"var(--font-sans)" }}>
            <div style={{ background: "#131c19", border: "1px solid rgba(255,255,255,0.10)", borderRadius: 10, boxShadow: "0 24px 60px rgba(0,0,0,0.5)", overflow: "hidden" }}>
              <div style={{ padding: "12px 16px", display: "flex", gap: 10, alignItems:"center", borderBottom: "1px solid rgba(255,255,255,0.06)" }}>
                <I.search size={14} style={{ color: "#98a09d" }}/>
                <span style={{ fontFamily: "var(--font-mono)", fontSize: 13, color: "#e6ebe9" }}>render</span>
                <span style={{ width: 1.5, height: 14, background: "#144a46", animation: "blink 1s steps(2) infinite" }}/>
                <span style={{ marginLeft: "auto", fontSize: 10, color: "#6b7370", fontFamily: "var(--font-mono)" }}>4 results</span>
              </div>
              {[
                { p: "src/md/", n: "render.rs", t: "rs", hot: true },
                { p: "src/code/", n: "render_lsp.rs", t: "rs" },
                { p: "tests/", n: "md_render.rs", t: "rs" },
                { p: "design/handoff/", n: "render-spec.md", t: "md" },
              ].map((r, i) => (
                <div key={i} style={{ display:"flex", alignItems:"center", gap: 10, padding: "10px 16px",
                  background: r.hot ? "rgba(20,74,70,0.20)" : "transparent", color: "#e6ebe9", fontSize: 12 }}>
                  {r.t === "rs" ? <I.fileCode size={14}/> : <I.fileText size={14}/>}
                  <span style={{ color: "#e6ebe9", fontWeight: 500 }}><b style={{ color: "#5fdfb6" }}>render</b>{r.n.slice(6)}</span>
                  <span style={{ marginLeft: "auto", color: "#6b7370", fontFamily:"var(--font-mono)", fontSize: 10.5 }}>{r.p}</span>
                </div>
              ))}
              <div style={{ padding: "8px 16px", display:"flex", gap: 12, fontSize: 10, color: "#6b7370", borderTop: "1px solid rgba(255,255,255,0.04)", fontFamily:"var(--font-mono)" }}>
                <span>↑↓ select</span><span>↵ open</span><span>⇧↵ split</span><span style={{ marginLeft:"auto" }}>esc</span>
              </div>
            </div>
          </div>
        </DCArtboard>
        <DCArtboard id="fe-loading" label="Loading skeleton" width={300} height={420}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <div style={{ width: 240, background: "var(--panel)", borderRight: "1px solid var(--border)", height: "100%" }}>
              <div className="moai-brand">
                <img className="mascot" src="assets/moai-logo-3.png" alt=""/>
                <span className="name">MoAI Studio</span>
              </div>
              <div className="moai-side-section">Loading workspace…</div>
              <LoadingSkeleton/>
            </div>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="fe-empty" label="Empty · no workspace" width={400} height={440}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <EmptyState kind="no-workspace"/>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 4: Agent Dashboard === */}
      <DCSection id="agent" title="Agent Dashboard (V3-010)" subtitle="The Agentic-IDE differentiator — event timeline, cost, instructions graph">
        <DCArtboard id="ag-dark" label="Live timeline · dark" width={1280} height={620}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <AgentDashboard/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="ag-light" label="Live timeline · light" width={1280} height={620}>
          <MoAI mode="light" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <AgentDashboard/>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 5: Markdown / Code === */}
      <DCSection id="rw" title="Read surfaces · Markdown + Code (V3-006/007)" subtitle="@MX gutter · LSP diagnostics inline">
        <DCArtboard id="md-dark" label="Markdown · @MX gutter (dark)" width={760} height={620}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "SPEC-V3-006.md", active: true, icon: <I.fileText size={12}/> }]}>
              <div style={{ flex:1, display:"flex" }}><MarkdownViewer/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="md-light" label="Markdown · @MX gutter (light)" width={760} height={620}>
          <MoAI mode="light" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "SPEC-V3-006.md", active: true, icon: <I.fileText size={12}/> }]}>
              <div style={{ flex:1, display:"flex" }}><MarkdownViewer/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="cd-dark" label="Code Viewer · LSP diagnostic (dark)" width={760} height={520}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "tree.rs", active: true, dirty: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ flex:1, display:"flex" }}><CodeViewer/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 6: Git === */}
      <DCSection id="git" title="Git Management (V3-008)" subtitle="Status · staged · diff (split) · sign-off + pre-commit hook">
        <DCArtboard id="git-dark" label="Commit · diff split (dark)" width={1280} height={620}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <GitMgmt/>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 7: SPEC === */}
      <DCSection id="spec" title="SPEC Management · Kanban (V3-009)" subtitle="Draft → Planned → In dev → Done · AC pips per card">
        <DCArtboard id="spec-dark" label="Kanban · dark" width={1280} height={620}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <SpecKanban/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="spec-light" label="Kanban · light" width={1280} height={620}>
          <MoAI mode="light" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <SpecKanban/>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 8: States === */}
      <DCSection id="states" title="States — First-run · Loading · Error · Empty" subtitle="Mascot in the emotional zones; clean chrome elsewhere.">
        <DCArtboard id="st-firstrun" label="First-run onboarding" width={760} height={580}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <FirstRun/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="st-error" label="Error banner · code viewer" width={760} height={460}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "persist.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ display:"flex", flexDirection:"column", flex:1 }}>
                <ErrorBanner/>
                <div style={{ flex:1, display:"flex" }}><CodeViewer/></div>
              </div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="st-empty-file" label="Empty · no file" width={620} height={420}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <EmptyState kind="no-file"/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="st-empty-spec" label="Empty · no specs yet" width={620} height={420}>
          <MoAI mode="light" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <EmptyState kind="no-specs"/>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 9: Web browser === */}
      <DCSection id="web" title="Web Browser (V3-007 · Tier 2)" subtitle="Dev-server auto-detect · HMR badge in URL bar">
        <DCArtboard id="web-dark" label="Dev preview · dark" width={780} height={560}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <WebBrowser/>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 10: Agent Running header variants === */}
      <DCSection id="agent-headers" title="Agent activity · header variants" subtitle="Subtle · balanced · prominent — pick where Agent surfaces.">
        <DCArtboard id="ah-subtle" label="Subtle (status bar dot only)" width={1100} height={120}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={true} showAgent={false}>
            <div style={{ flex:1, background: "var(--bg)", display:"grid", placeItems:"center", color: "var(--fg-3)", fontSize: 11 }}>chrome only — agent indicator lives in status bar bottom-right</div>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="ah-balanced" label="Balanced (header pill)" width={1100} height={120}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={true} showAgent={true}>
            <div style={{ flex:1, background: "var(--bg)", display:"grid", placeItems:"center", color: "var(--fg-3)", fontSize: 11 }}>balanced — pill in top bar + status bar number</div>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="ah-prominent" label="Prominent (live timeline strip)" width={1100} height={170}>
          <MoAI mode="dark" accent={accent} density={density} sidebar="none" showStatus={true} showAgent={true}>
            <div style={{ display:"flex", alignItems:"center", gap: 14, padding: "8px 14px", background: "linear-gradient(90deg, rgba(20,74,70,0.32) 0%, rgba(20,74,70,0.05) 100%)", borderBottom: "1px solid var(--border)", fontFamily: "var(--font-mono)", fontSize: 10.5, color: "var(--fg-2)", overflow:"hidden", whiteSpace: "nowrap" }}>
              <span className="moai-agent-pill"><span className="moai-agent-dot"/> live</span>
              <span style={{ color: "var(--fg-3)" }}>13:45:36</span><I.wrench size={11}/><span>Bash · cargo test</span>
              <span style={{ color: "var(--fg-3)" }}>13:45:40</span><I.check size={11} style={{ color: "var(--mint)" }}/><span>17 passed · 1 failed</span>
              <span style={{ color: "var(--fg-3)" }}>13:45:42</span><I.wrench size={11}/><span>Edit · 3 hunks</span>
              <span style={{ marginLeft:"auto", color: "var(--accent)" }}>⌘⇧A · open dashboard</span>
            </div>
            <div style={{ flex:1, background: "var(--bg)", display:"grid", placeItems:"center", color: "var(--fg-3)", fontSize: 11 }}>prominent — agent timeline lives above tabs</div>
          </MoAI>
        </DCArtboard>
      </DCSection>
      {/* === Section 11: Command surfaces (Round 2) === */}
      <DCSection id="palettes" title="Command surfaces — palettes &amp; bars" subtitle="⌘P · ⌘⇧P · ⌘F · /moai slash dispatch. Floating overlays on top of the IDE.">
        <DCArtboard id="cmd-p" label="⌘P · File quick-open" width={840} height={580}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "render.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ position:"relative", flex:1, display:"flex" }}>
                <CodeStub/>
                <CmdPalette query="spec/v3"/>
              </div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="cmd-shift-p" label="⌘⇧P · Command palette" width={840} height={580}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "render.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ position:"relative", flex:1, display:"flex" }}>
                <CodeStub/>
                <CommandPalette query=">git"/>
              </div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="slash" label="/moai · agent slash" width={760} height={460}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "Agent", active: true, icon: <I.agent size={12}/> }]}>
              <div style={{ position:"relative", flex:1, display:"flex", background:"var(--bg)" }}>
                <SlashBar/>
              </div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="cmd-f" label="⌘F · find / replace" width={900} height={460}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "render.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ flex:1, display:"flex", minWidth:0 }}><FindReplace/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 12: Editor popovers === */}
      <DCSection id="popovers" title="Editor popovers — LSP &amp; @MX" subtitle="Hover-anchored detail surfaces with footer keybinds.">
        <DCArtboard id="lsp-hover" label="LSP hover · rust-analyzer" width={760} height={520}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "render.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ flex:1, display:"flex", minWidth:0 }}><LspHover/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="mx-pop" label="@MX:ANCHOR · tag detail" width={760} height={520}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "focus.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ flex:1, display:"flex", minWidth:0 }}><MXPopover tag="ANCHOR"/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 13: Git deeper === */}
      <DCSection id="git-deep" title="Git — 3-way merge conflict" subtitle="Base · Ours · Theirs — accept-direction or stage chunks individually.">
        <DCArtboard id="git-merge" label="Conflict · persist.rs" width={1280} height={580}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "persist.rs · MERGING", active: true, icon: <I.branch size={12}/> }]}>
              <div style={{ flex:1, display:"flex", minWidth:0 }}><MergeDiff/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 14: SPEC deeper === */}
      <DCSection id="spec-deep" title="SPEC · Sprint Contract" subtitle="The approval surface — moai's promise to ship.">
        <DCArtboard id="sprint" label="Sprint contract · review &amp; approve" width={520} height={680}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <SprintPanel/>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 15: Settings === */}
      <DCSection id="settings" title="Settings · Preferences modal" subtitle="Two-pane modal — left nav, right surface. ⌘, opens.">
        <DCArtboard id="set-appearance" label="Appearance" width={1080} height={680}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "render.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ position:"relative", flex:1, display:"flex" }}>
                <CodeStub/>
                <SettingsModal section="appearance"/>
              </div>
            </Surface>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="set-keyboard" label="Keyboard shortcuts" width={1080} height={680}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "render.rs", active: true, icon: <I.fileCode size={12}/> }]}>
              <div style={{ position:"relative", flex:1, display:"flex" }}>
                <CodeStub/>
                <SettingsModal section="keyboard"/>
              </div>
            </Surface>
          </MoAI>
        </DCArtboard>
      </DCSection>

      {/* === Section 16: System banners === */}
      <DCSection id="banners" title="System banners — non-blocking states" subtitle="Crash · update · LSP/PTY starting. Top of canvas, dismissable.">
        <DCArtboard id="ban-crash" label="Agent crashed" width={1080} height={120}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <CrashBanner/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="ban-update" label="Update available" width={1080} height={120}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <UpdateBanner/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="ban-lsp" label="rust-analyzer starting" width={1080} height={120}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <LspStarting/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="ban-pty" label="Terminal spawning" width={1080} height={120}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <PtyStarting/>
          </MoAI>
        </DCArtboard>
        <DCArtboard id="ws-switch" label="Workspace switching" width={760} height={520}>
          <MoAI mode={mode} accent={accent} density={density} sidebar="none" showStatus={false} showAgent={false}>
            <Surface tabs={[{ name: "render.rs", icon: <I.fileCode size={12}/> }]}>
              <div style={{ flex:1, display:"flex", minWidth:0 }}><WorkspaceSwitching/></div>
            </Surface>
          </MoAI>
        </DCArtboard>
      </DCSection>
    </DesignCanvas>
  </>);
}

ReactDOM.createRoot(document.getElementById("root")).render(<App/>);
