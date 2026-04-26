// moai-revisions.jsx — Round 2 surfaces & states (P1)
// 9 new surfaces: CmdP, CmdShiftP, CmdF, LSP Hover, MX Popover,
//                 3-way Merge, Sprint Contract, /moai Slash, Settings
// 5 new states:   Crash, Update, LSP Starting, PTY Starting, Workspace Switch

const { I } = window;

/* ─── Cmd+P · File Quick Open ─────────────────────────────────────── */
function CmdPalette({ query = "spec/v3", count = 12, total = 234 }) {
  const items = [
    { sel: true, name: "src/main.rs", path: "src", meta: ".rs · 456 lines", icon: <I.fileCode size={14}/> },
    { name: "src/spec/v3-005.rs", path: "src/spec", meta: ".rs · 234 lines", icon: <I.fileCode size={14}/>, highlight: "spec/v3" },
    { name: "src/spec/v3-006.rs", path: "src/spec", meta: ".rs · 189 lines", icon: <I.fileCode size={14}/>, highlight: "spec/v3" },
    { name: ".moai/specs/SPEC-V3-005/spec.md", path: ".moai/specs", meta: ".md · 12kb", icon: <I.fileText size={14}/>, highlight: "v3" },
    { name: ".moai/specs/SPEC-V3-006/spec.md", path: ".moai/specs", meta: ".md · 9.4kb", icon: <I.fileText size={14}/>, highlight: "v3" },
    { name: "tests/spec_v3_runner.rs", path: "tests", meta: ".rs · 78 lines", icon: <I.fileCode size={14}/>, highlight: "spec_v3" },
    { name: "tokens.json", path: ".moai/design", meta: ".json · 8.2kb", icon: <I.fileText size={14}/> },
    { name: "Cargo.toml", path: "/", meta: ".toml · 3.1kb", icon: <I.fileText size={14}/> },
  ];
  return (
    <div className="ovl-scrim">
      <div className="pal" onClick={(e)=>e.stopPropagation()}>
        <div className="pal-input">
          <I.search size={14}/>
          <input defaultValue={query} placeholder="Search files by name…" autoFocus={false}/>
          <span className="pal-count">{count}/{total}</span>
        </div>
        <div className="pal-section">Files in workspace</div>
        <div className="pal-list">
          {items.map((it, i) => (
            <div key={i} className={`pal-row ${it.sel ? "sel" : ""}`}>
              <span className="pal-ic">{it.icon}</span>
              <span className="pal-nm">
                {it.highlight
                  ? it.name.split(it.highlight).map((p, j, arr) => (
                      <React.Fragment key={j}>{p}{j < arr.length - 1 ? <em>{it.highlight}</em> : null}</React.Fragment>
                    ))
                  : it.name}
                <span className="pal-path">— {it.path}</span>
              </span>
              <span className="pal-meta">{it.meta}</span>
            </div>
          ))}
        </div>
        <div className="pal-foot">
          <span><kbd className="kbd">↑↓</kbd> navigate</span>
          <span><kbd className="kbd">↵</kbd> open</span>
          <span><kbd className="kbd">⌘↵</kbd> split right</span>
          <span style={{ marginLeft: "auto" }}><kbd className="kbd">esc</kbd> close</span>
        </div>
      </div>
    </div>
  );
}

/* ─── Cmd+Shift+P · Command Palette ───────────────────────────────── */
function CommandPalette({ query = ">git" }) {
  return (
    <div className="ovl-scrim">
      <div className="pal" onClick={(e)=>e.stopPropagation()}>
        <div className="pal-input">
          <I.terminal size={14}/>
          <input defaultValue={query} placeholder="> type a command…"/>
          <span className="pal-count">14/96</span>
        </div>
        <div className="pal-section">Recent</div>
        <div className="pal-list" style={{ maxHeight: "none" }}>
          <div className="pal-row sel">
            <span className="pal-ic"><I.branch size={14}/></span>
            <span className="pal-nm">Git: Commit Staged Changes <span className="pal-path">— composer + sign-off</span></span>
            <span className="kbd">⌘K ⌘C</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.fileCode size={14}/></span>
            <span className="pal-nm">View: Split Pane <em>—</em> Horizontal</span>
            <span className="kbd">⌘\</span>
          </div>
          <div className="pal-section">All Commands · Git</div>
          <div className="pal-row">
            <span className="pal-ic"><I.branch size={14}/></span>
            <span className="pal-nm">Git: Pull from origin/main</span>
            <span className="kbd">⌘K ⌘P</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.branch size={14}/></span>
            <span className="pal-nm">Git: Create Branch from current</span>
            <span className="pal-meta">—</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.branch size={14}/></span>
            <span className="pal-nm">Git: Discard All Changes</span>
            <span className="pal-meta">⚠ destructive</span>
          </div>
          <div className="pal-section">All Commands · Workspace</div>
          <div className="pal-row">
            <span className="pal-ic"><I.folder size={14}/></span>
            <span className="pal-nm">Workspace: Add Folder…</span>
            <span className="pal-meta">—</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.settings size={14}/></span>
            <span className="pal-nm">Preferences: Open Settings…</span>
            <span className="kbd">⌘,</span>
          </div>
        </div>
        <div className="pal-foot">
          <span><kbd className="kbd">↑↓</kbd> navigate</span>
          <span><kbd className="kbd">↵</kbd> run</span>
          <span style={{ marginLeft: "auto", color: "var(--accent)" }}>96 commands</span>
        </div>
      </div>
    </div>
  );
}

/* ─── /moai Slash Command Bar ─────────────────────────────────────── */
function SlashBar({ query = "/moai " }) {
  return (
    <div className="ovl-scrim" style={{ paddingTop: 60 }}>
      <div className="pal" style={{ width: 540 }} onClick={(e)=>e.stopPropagation()}>
        <div className="pal-input">
          <span style={{ color: "var(--accent)", fontFamily:"var(--font-mono)", fontWeight: 700 }}>/</span>
          <input defaultValue={query} placeholder="/ enter a slash command…"/>
          <span className="pal-count">7</span>
        </div>
        <div className="pal-section">moai · agent commands</div>
        <div className="pal-list">
          <div className="pal-row sel">
            <span className="pal-ic" style={{ color: "var(--accent)" }}><I.zap size={14}/></span>
            <span className="pal-nm"><em>/moai</em> plan <span className="pal-path">— draft sprint plan from active SPEC</span></span>
            <span className="pal-meta">3 args</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.zap size={14}/></span>
            <span className="pal-nm"><em>/moai</em> run <span className="pal-path">— execute approved sprint contract</span></span>
            <span className="pal-meta">1 arg</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.zap size={14}/></span>
            <span className="pal-nm"><em>/moai</em> sync <span className="pal-path">— sync .moai/ with origin</span></span>
            <span className="pal-meta">—</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.zap size={14}/></span>
            <span className="pal-nm"><em>/moai</em> spec <span className="pal-path">— scaffold new SPEC under .moai/specs</span></span>
            <span className="pal-meta">2 args</span>
          </div>
          <div className="pal-row">
            <span className="pal-ic"><I.zap size={14}/></span>
            <span className="pal-nm"><em>/moai</em> review <span className="pal-path">— review against acceptance criteria</span></span>
            <span className="pal-meta">—</span>
          </div>
        </div>
        <div className="pal-foot">
          <span><kbd className="kbd">tab</kbd> autocomplete</span>
          <span><kbd className="kbd">↵</kbd> dispatch</span>
        </div>
      </div>
    </div>
  );
}

/* ─── Cmd+F · Find / Replace ─────────────────────────────────────── */
function FindReplace({ surface = <CodeStub/> }) {
  return (
    <div style={{ position: "relative", height: "100%" }}>
      {surface}
      <div className="find">
        <div className="row">
          <I.search size={12}/>
          <input defaultValue="function" />
          <span className="cnt">12 of 45</span>
          <button className="ibtn on" title="Match case">Aa</button>
          <button className="ibtn" title="Whole word">W</button>
          <button className="ibtn" title="Regex">.*</button>
          <button className="ibtn close" title="Close"><I.x size={12}/></button>
        </div>
        <div className="row">
          <I.zap size={12} />
          <input defaultValue="newFunction"/>
        </div>
        <div className="actrow">
          <button className="btn">Replace</button>
          <button className="btn">Replace All</button>
          <button className="btn">↑ Prev</button>
          <button className="btn pri">↓ Next</button>
        </div>
      </div>
    </div>
  );
}

function CodeStub() {
  return (
    <div className="code">
      <div className="gut">
        {[12,13,14,15,16,17,18,19,20,21,22,23].map(n => <span key={n} className="ln">{n}</span>)}
      </div>
      <div className="src">
        <span className="kw">pub</span> <span className="kw">fn</span> <span className="fn">new_file</span>(path: <span className="ty">&Path</span>) -&gt; <span className="ty">Result</span>&lt;<span className="ty">File</span>&gt; {"{"}
        <br/>{"    "}<span className="kw">let</span> mut f = <span className="fn">File</span>::<span className="fn">create</span>(path)?;
        <br/>{"    "}f.<span className="fn">write_all</span>(<span className="st">b""</span>)?;
        <br/>{"    "}<span className="kw">Ok</span>(f)
        <br/>{"}"}
        <br/>
        <br/><span className="kw">pub</span> <span className="kw">fn</span> <span className="fn">function</span>_one(x: <span className="ty">i32</span>) -&gt; <span className="ty">i32</span> {"{ x + 1 }"}
        <br/><span className="kw">pub</span> <span className="kw">fn</span> <span className="fn">function</span>_two(y: <span className="ty">i32</span>) -&gt; <span className="ty">i32</span> {"{ y * 2 }"}
        <br/><span className="kw">pub</span> <span className="kw">fn</span> <span className="fn">function</span>_three() -&gt; <span className="ty">String</span> {"{ \"hi\".to_string() }"}
      </div>
    </div>
  );
}

/* ─── LSP Hover Popover ──────────────────────────────────────────── */
function LspHover() {
  return (
    <div style={{ position: "relative", height: "100%" }}>
      <CodeStub/>
      <div className="lspop" style={{ top: 80, left: 110 }}>
        <div className="ed-anchor"/>
        <div className="sig">
          <span className="kw">pub fn</span> <span className="fn">new_file</span>(path: <span className="ty">&Path</span>) -&gt; <span className="ty">Result</span>&lt;<span className="ty">File</span>, <span className="ty">io::Error</span>&gt;
        </div>
        <div className="doc">
          <p>Creates a new file at <strong>path</strong> in the current workspace.</p>
          <p>Returns an open <code>File</code> handle on success, or an <code>io::Error</code> if the parent directory does not exist or the workspace is read-only.</p>
          <div className="pdef">
            <span className="pn">@param path</span><span className="pd">target file path (relative or absolute)</span>
            <span className="pn">@returns</span><span className="pd">Result&lt;File, io::Error&gt;</span>
            <span className="pn">@since</span><span className="pd">v0.1.0</span>
          </div>
        </div>
        <div className="foot">
          <span><kbd>F12</kbd> Go to def</span>
          <span><kbd>⇧F12</kbd> Find refs</span>
          <span style={{ marginLeft: "auto" }}>rust-analyzer</span>
        </div>
      </div>
    </div>
  );
}

/* ─── MX Tag Popover ─────────────────────────────────────────────── */
function MXPopover({ tag = "ANCHOR" }) {
  const tagClass = tag === "WARN" ? "warn" : tag === "TODO" ? "todo" : "";
  return (
    <div style={{ position: "relative", height: "100%" }}>
      <div className="md md-frame" style={{ padding: 0, height: "100%" }}>
        <div className="md-gutter">
          <div className="gtag" style={{ marginTop: 28 }}>@MX</div>
          <div className="gtag" style={{ background: "var(--accent-soft)", padding: "2px 4px", borderRadius: 3 }}>:ANCHOR</div>
          <div className="gtag">@MX</div>
          <div className="gtag">@MX</div>
          <div className="gtag">:WARN</div>
          <div className="gtag">@MX</div>
        </div>
        <div className="md-scroll">
          <div className="md">
            <h2>refresh_pane_focus()</h2>
            <p>Refreshes the active pane's focus state, restoring any previously focused tab.</p>
            <p>This function is called whenever the user switches tabs or the workspace is reloaded from persistence.</p>
            <p style={{ color: "var(--fg-3)" }}>Defined at <code>src/panes/focus.rs:142</code> · 5 callers across the codebase.</p>
          </div>
        </div>
      </div>
      <div className="mxpop" style={{ top: 88, left: 100 }}>
        <div className="hdr">
          <span className={`tag ${tagClass}`}>@MX:{tag}</span>
          <span className="id">refresh_pane_focus</span>
        </div>
        <div className="body">
          <div className="meta">
            <span className="k">fan_in</span><span className="v warn">5 (high-risk)</span>
            <span className="k">spec</span><span className="v acc">SPEC-V3-003</span>
            <span className="k">since</span><span className="v">v0.1.0</span>
            <span className="k">owner</span><span className="v">@goos</span>
          </div>
          <p><strong>Reason:</strong> API contract — this function is called from 5 locations including persistence, tab-switch, and split-resize handlers. Any signature change must update all call sites.</p>
          <p style={{ color: "var(--fg-3)", fontSize: 11 }}>"Be careful with changes — refactor would break the persistence schema."</p>
        </div>
        <div className="actions">
          <button className="btn"><I.search size={12}/> View 5 usages</button>
          <button className="btn pri"><I.fileText size={12}/> Open SPEC</button>
        </div>
      </div>
    </div>
  );
}

/* ─── 3-way Merge Conflict Diff ──────────────────────────────────── */
function MergeDiff() {
  return (
    <div className="merge">
      <div className="merge-hd">
        <I.branch size={13}/>
        <span className="file">src/panes/persist.rs</span>
        <span className="badge">CONFLICT</span>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: 10.5, color: "var(--fg-3)" }}>3 chunks</span>
        <div className="actions">
          <button className="btn"><I.check size={11}/> Accept Ours</button>
          <button className="btn"><I.check size={11}/> Accept Theirs</button>
          <button className="btn">Accept Both</button>
          <button className="btn pri"><I.check size={11}/> Mark Resolved</button>
        </div>
      </div>
      <div className="merge-cols">
        <div className="merge-col base">
          <h5>Base <span className="br">main · 4f3a2c1</span></h5>
          <div className="merge-body">
            <div className="ln ctx"><span className="nr">38</span><span>pub fn save(&self) {"{"}</span></div>
            <div className="ln ctx"><span className="nr">39</span><span>{"  "}let path = self.path();</span></div>
            <div className="ln ctx"><span className="nr">40</span><span>{"  "}let data = serde_json::to_vec(</span></div>
            <div className="ln ctx"><span className="nr">41</span><span>{"    "}&self.snapshot(),</span></div>
            <div className="ln ctx"><span className="nr">42</span><span>{"  "}).unwrap();</span></div>
            <div className="ln ctx"><span className="nr">43</span><span>{"  "}fs::write(path, data)</span></div>
            <div className="ln ctx"><span className="nr">44</span><span>{"    "}.expect("save failed");</span></div>
            <div className="ln ctx"><span className="nr">45</span><span>{"}"}</span></div>
          </div>
        </div>
        <div className="merge-col ours">
          <h5>Ours <span className="br">feature/persist-async</span></h5>
          <div className="merge-body">
            <div className="ln ctx"><span className="nr">38</span><span>pub async fn save(&self) {"{"}</span></div>
            <div className="ln ctx"><span className="nr">39</span><span>{"  "}let path = self.path();</span></div>
            <div className="ln ctx"><span className="nr">40</span><span>{"  "}let data = serde_json::to_vec(</span></div>
            <div className="ln ctx"><span className="nr">41</span><span>{"    "}&self.snapshot(),</span></div>
            <div className="ln ctx"><span className="nr">42</span><span>{"  "}).unwrap();</span></div>
            <div className="conflict-mark">{"<<<<<<< OURS"}</div>
            <div className="ln add"><span className="nr">43</span><span>{"  "}tokio::fs::write(path, data)</span></div>
            <div className="ln add"><span className="nr">44</span><span>{"    "}.await</span></div>
            <div className="ln add"><span className="nr">45</span><span>{"    "}.map_err(Into::into)</span></div>
            <div className="conflict-mark">{"========="}</div>
            <div className="ln ctx"><span className="nr">46</span><span>{"}"}</span></div>
          </div>
        </div>
        <div className="merge-col theirs">
          <h5>Theirs <span className="br">feature/persist-debounce</span></h5>
          <div className="merge-body">
            <div className="ln ctx"><span className="nr">38</span><span>pub fn save(&self) {"{"}</span></div>
            <div className="ln ctx"><span className="nr">39</span><span>{"  "}let path = self.path();</span></div>
            <div className="ln ctx"><span className="nr">40</span><span>{"  "}let data = serde_json::to_vec(</span></div>
            <div className="ln ctx"><span className="nr">41</span><span>{"    "}&self.snapshot(),</span></div>
            <div className="ln ctx"><span className="nr">42</span><span>{"  "}).unwrap();</span></div>
            <div className="conflict-mark">{"<<<<<<< THEIRS"}</div>
            <div className="ln cnf"><span className="nr">43</span><span>{"  "}self.debouncer</span></div>
            <div className="ln cnf"><span className="nr">44</span><span>{"    "}.queue(path, data);</span></div>
            <div className="ln del"><span className="nr">45</span><span>{"  "}// fs::write removed</span></div>
            <div className="conflict-mark">{">>>>>>> END"}</div>
            <div className="ln ctx"><span className="nr">46</span><span>{"}"}</span></div>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ─── Sprint Contract Panel ──────────────────────────────────────── */
function SprintPanel() {
  return (
    <div style={{ height: "100%", padding: 28, background: "var(--bg)", display:"grid", placeItems: "center" }}>
      <div className="sprint">
        <div className="hdr">
          <div className="tit">Sprint Contract · v1.0.x</div>
          <div className="sub">SPEC-V3-005 · Review &amp; Approve</div>
        </div>
        <div className="body">
          <div className="lbl">Priority dimension</div>
          <div className="pri-pill"><span className="dot"/> Design Quality &amp; AC parity</div>

          <div className="lbl">Acceptance checklist</div>
          <div className="ac-list">
            <div className="ac-row done">
              <span className="ck"/>
              <span className="lab"><span className="id">AC-FE-1</span>FsNode tree renders with 26px row</span>
              <span className="est">~1h</span>
            </div>
            <div className="ac-row done">
              <span className="ck"/>
              <span className="lab"><span className="id">AC-FE-4</span>Git status badge color/contrast</span>
              <span className="est">~30m</span>
            </div>
            <div className="ac-row">
              <span className="ck"/>
              <span className="lab"><span className="id">AC-FE-7</span>Fuzzy search palette + highlight</span>
              <span className="est">~2h</span>
            </div>
            <div className="ac-row">
              <span className="ck"/>
              <span className="lab"><span className="id">AC-FE-12</span>Expand/collapse animation</span>
              <span className="est">~45m</span>
            </div>
            <div className="ac-row fail">
              <span className="ck"/>
              <span className="lab"><span className="id">AC-FE-6</span>Drag-and-drop reorder</span>
              <span className="est">deferred</span>
            </div>
          </div>

          <div className="lbl">Pass score</div>
          <div className="meter"><div className="fill" style={{ width: "82%" }}/></div>
          <div className="meter-row">
            <span>0.82 / 1.0</span>
            <span className="pass">≥ 0.75 PASS</span>
          </div>

          <div className="actions">
            <button className="btn sec">Edit AC</button>
            <button className="btn pri">Approve &amp; Run</button>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ─── Settings / Preferences Modal ───────────────────────────────── */
function SettingsModal({ section = "appearance" }) {
  return (
    <div className="ovl-scrim" style={{ paddingTop: 30, alignItems: "center" }}>
      <div className="settings" onClick={(e)=>e.stopPropagation()}>
        <div className="nav">
          <div className="sec">Workspace</div>
          <div className={`ni ${section==="appearance"?"active":""}`}><I.settings size={13}/> Appearance</div>
          <div className={`ni ${section==="editor"?"active":""}`}><I.fileCode size={13}/> Editor</div>
          <div className={`ni ${section==="terminal"?"active":""}`}><I.terminal size={13}/> Terminal</div>
          <div className={`ni ${section==="git"?"active":""}`}><I.branch size={13}/> Git</div>
          <div className="sec">Productivity</div>
          <div className={`ni ${section==="keyboard"?"active":""}`}><I.terminal size={13}/> Keyboard</div>
          <div className={`ni`}><I.zap size={13}/> Extensions</div>
          <div className="sec">System</div>
          <div className={`ni`}><I.alertTri size={13}/> About</div>
        </div>
        <div className="pane">
          <div className="pane-hd">
            <h4>{section === "keyboard" ? "Keyboard Shortcuts" : section === "appearance" ? "Appearance" : "Settings"}</h4>
            <input type="text" placeholder="Search settings…" style={{ minWidth: 220, marginLeft: 14 }}/>
            <button className="x"><I.x size={14}/></button>
          </div>
          <div className="pane-bd">
            {section === "appearance" && <AppearancePane/>}
            {section === "keyboard" && <KeyboardPane/>}
          </div>
        </div>
      </div>
    </div>
  );
}

function AppearancePane() {
  return (
    <>
      <div className="grp">
        <h5>Theme</h5>
        <div className="opt">
          <div className="lab">Mode<div className="help">Light, dark, or follow system</div></div>
          <div className="ctl">
            <div className="seg">
              <button>Light</button>
              <button className="on">Dark</button>
              <button>System</button>
            </div>
          </div>
        </div>
        <div className="opt">
          <div className="lab">Accent color<div className="help">Primary color for buttons, focus, highlights</div></div>
          <div className="ctl">
            <div className="swatch-row">
              <span className="swatch on" style={{ background: "#144a46" }}/>
              <span className="swatch" style={{ background: "#2563EB" }}/>
              <span className="swatch" style={{ background: "#6a4cc7" }}/>
              <span className="swatch" style={{ background: "#06B6D4" }}/>
            </div>
          </div>
        </div>
        <div className="opt">
          <div className="lab">Density<div className="help">Row height for lists and trees</div></div>
          <div className="ctl">
            <div className="seg">
              <button className="on">Comfortable</button>
              <button>Compact</button>
            </div>
          </div>
        </div>
      </div>
      <div className="grp">
        <h5>Typography</h5>
        <div className="opt">
          <div className="lab">UI font</div>
          <div className="ctl"><select defaultValue="pretendard"><option value="pretendard">Pretendard Variable</option><option>System UI</option></select></div>
        </div>
        <div className="opt">
          <div className="lab">Mono font</div>
          <div className="ctl"><select defaultValue="jetbrains"><option value="jetbrains">JetBrains Mono</option><option>SF Mono</option><option>Cascadia Code</option></select></div>
        </div>
        <div className="opt">
          <div className="lab">Font size</div>
          <div className="ctl">
            <div className="seg"><button>12</button><button className="on">13</button><button>14</button><button>16</button></div>
          </div>
        </div>
      </div>
      <div className="grp">
        <h5>Layout</h5>
        <div className="opt">
          <div className="lab">Sidebar position</div>
          <div className="ctl"><div className="seg"><button className="on">Left</button><button>Right</button></div></div>
        </div>
        <div className="opt">
          <div className="lab">Reduce motion<div className="help">Honor system prefers-reduced-motion</div></div>
          <div className="ctl"><div className="toggle on"/></div>
        </div>
      </div>
    </>
  );
}

function KeyboardPane() {
  const rows = [
    ["Open File…", "⌘", "P"],
    ["Command Palette", "⌘", "⇧", "P"],
    ["Find in File", "⌘", "F"],
    ["Replace in File", "⌘", "H"],
    ["New Terminal Tab", "⌘", "T"],
    ["Split Pane Horizontal", "⌘", "\\"],
    ["Split Pane Vertical", "⌘", "⇧", "\\"],
    ["Close Pane", "⌘", "W"],
    ["Focus Next Pane", "⌘", "]"],
    ["Focus Prev Pane", "⌘", "["],
    ["Open Settings", "⌘", ","],
    ["Toggle Sidebar", "⌘", "B"],
    ["Toggle Agent Panel", "⌘", "J"],
    ["Commit Staged", "⌘", "K", "⌘", "C"],
  ];
  return (
    <div className="grp">
      <h5>Default bindings</h5>
      <table className="kb-table">
        <tbody>
          {rows.map((r, i) => (
            <tr key={i}>
              <td>{r[0]}</td>
              <td>{r.slice(1).map((k, j) => <kbd key={j}>{k}</kbd>)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/* ─── States: Crash / Update / LSP / PTY / Workspace ──────────────── */
function CrashBanner() {
  return (
    <div className="banner crash">
      <I.alertTri size={16}/>
      <div className="body">
        <strong>Agent crashed</strong> · core dumped during /moai run · child PID 4892 sigsegv
        <div className="meta">.moai/logs/agent-2026-04-25-14-12.log · last alive 12s ago</div>
      </div>
      <div className="actions">
        <button className="btn">View Log</button>
        <button className="btn">Report</button>
        <button className="btn pri">Restart Agent</button>
      </div>
    </div>
  );
}

function UpdateBanner() {
  return (
    <div className="banner update">
      <I.zap size={16}/>
      <div className="body">
        <strong>Update v0.2.0 available</strong> · LSP performance + 3-way merge fixes
        <div className="meta">52.3 MB · published 2h ago · changelog →</div>
      </div>
      <div className="actions">
        <button className="btn">Later</button>
        <button className="btn pri">Install &amp; Restart</button>
      </div>
    </div>
  );
}

function LspStarting() {
  return (
    <div className="banner starting">
      <I.zap size={16}/>
      <div className="body">
        <strong style={{ color: "var(--accent)" }}>rust-analyzer</strong> initializing<span className="dots"><span/><span/><span/></span>
        <div className="meta">indexing 1,284 source files · estimated 3s remaining</div>
      </div>
      <div className="actions">
        <button className="btn">Disable for project</button>
      </div>
    </div>
  );
}

function PtyStarting() {
  return (
    <div className="banner starting">
      <I.terminal size={16}/>
      <div className="body">
        <strong style={{ color: "var(--accent)" }}>Terminal</strong> spawning<span className="dots"><span/><span/><span/></span>
        <div className="meta">/bin/zsh · cwd ~/MoAI/moai-studio · pty allocated</div>
      </div>
      <div className="actions">
        <button className="btn">Cancel</button>
      </div>
    </div>
  );
}

function WorkspaceSwitching({ from = "moai-studio", to = "playground-rs" }) {
  return (
    <div style={{ position: "relative", height: "100%" }}>
      <div className="code">
        <div className="gut">{[1,2,3,4,5,6,7,8].map(n => <span key={n} className="ln">{n}</span>)}</div>
        <div className="src" style={{ opacity: 0.4 }}>// previous workspace fading…</div>
      </div>
      <div className="wsw">
        <img src="assets/moai-logo-3.png" alt=""/>
        <div className="lab">Switching workspace</div>
        <div className="meta">{from} → {to}</div>
        <div style={{ display:"flex", gap: 4, marginTop: 6 }}>
          <span style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--accent)" }}/>
          <span style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--accent-soft)" }}/>
          <span style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--accent-soft)" }}/>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, {
  CmdPalette, CommandPalette, SlashBar, FindReplace, CodeStub,
  LspHover, MXPopover, MergeDiff, SprintPanel, SettingsModal,
  CrashBanner, UpdateBanner, LspStarting, PtyStarting, WorkspaceSwitching,
});
