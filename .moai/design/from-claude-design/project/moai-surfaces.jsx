// MoAI Studio — surface mocks (Terminal, File Explorer, Code, Markdown, Agent, Git, SPEC, Web Browser)

const { I } = window;

// ─── File Tree (used in sidebar) ─────────────────────────────────────
function FileTree({ items, density = "comfortable" }) {
  return (
    <div className="ftree" style={{ "--row-h": density === "compact" ? "22px" : "26px" }}>
      {items.map((it, i) => <FileRow key={i} {...it} />)}
    </div>
  );
}
function FileRow({ name, lvl = 0, kind = "file", open, active, gs, ext }) {
  const Icon = kind === "folder" ? (open ? I.folderOpen : I.folder) : extToIcon(ext);
  return (
    <div className={"frow " + (open ? "open " : "") + (active ? "active" : "")} style={{ "--lvl": lvl }}>
      {kind === "folder" ? <I.chev size={10}/> : <span style={{width:10}}/>}
      <Icon size={14}/>
      <span className="nm">{name}</span>
      {gs && <span className={"gs " + gs}>{gs === "Q" ? "?" : gs}</span>}
    </div>
  );
}
function extToIcon(ext){
  if (ext === "rs" || ext === "ts" || ext === "tsx" || ext === "js") return I.fileCode;
  if (ext === "md") return I.fileText;
  return I.file;
}

const SAMPLE_TREE = [
  { name: ".moai", kind: "folder", open: true, lvl: 0 },
  { name: "specs", kind: "folder", open: true, lvl: 1 },
  { name: "SPEC-V3-005.md", kind: "file", ext: "md", lvl: 2, gs: "M" },
  { name: "SPEC-V3-006.md", kind: "file", ext: "md", lvl: 2, active: true },
  { name: "config.yaml", kind: "file", lvl: 1 },
  { name: "src", kind: "folder", open: true, lvl: 0 },
  { name: "panes", kind: "folder", open: true, lvl: 1 },
  { name: "tree.rs", kind: "file", ext: "rs", lvl: 2, gs: "M" },
  { name: "tab_bar.rs", kind: "file", ext: "rs", lvl: 2 },
  { name: "divider.rs", kind: "file", ext: "rs", lvl: 2, gs: "A" },
  { name: "surfaces", kind: "folder", lvl: 1 },
  { name: "lib.rs", kind: "file", ext: "rs", lvl: 1 },
  { name: "main.rs", kind: "file", ext: "rs", lvl: 1 },
  { name: "tests", kind: "folder", lvl: 0 },
  { name: "Cargo.toml", kind: "file", lvl: 0, gs: "M" },
  { name: "README.md", kind: "file", ext: "md", lvl: 0 },
  { name: ".gitignore", kind: "file", lvl: 0, gs: "Q" },
];

// ─── Terminal ────────────────────────────────────────────────────────
function Terminal({ variant = "default" }) {
  return (
    <div className="term">
      <div><span className="cm"># cargo test in libghostty-vt</span></div>
      <div><span className="pr">~/moai-studio</span> <span className="br">main</span> <span className="pr">$</span> cargo test --workspace</div>
      <div className="cm">   Compiling moai-core v0.3.0 (panes)</div>
      <div className="cm">   Compiling moai-surfaces v0.3.0</div>
      <div>    Finished <span className="ok">test</span> [unoptimized + debuginfo] target(s) in 4.21s</div>
      <div>     Running unittests src/lib.rs</div>
      <div></div>
      <div>running 18 tests</div>
      <div>test panes::tree::split_horizontal ... <span className="ok">ok</span></div>
      <div>test panes::tree::split_vertical ... <span className="ok">ok</span></div>
      <div>test panes::tree::min_size_constraint ... <span className="ok">ok</span></div>
      <div>test panes::divider::drag_resize ... <span className="ok">ok</span></div>
      <div>test panes::tab::last_focused ... <span className="ok">ok</span></div>
      <div>test panes::persist::roundtrip ... <span className="wr">FAILED</span></div>
      <div></div>
      <div>failures:</div>
      <div className="er">---- panes::persist::roundtrip stdout ----</div>
      <div className="cm">  thread 'panes::persist::roundtrip' panicked at 'expected Some, got None'</div>
      <div className="cm">  src/panes/persist.rs:142:9</div>
      <div></div>
      <div>test result: <span className="er">FAILED</span>. 17 passed; 1 failed; 0 ignored</div>
      <div></div>
      <div><span className="pr">~/moai-studio</span> <span className="br">main</span> <span className="pr">$</span> <span className="cur"></span></div>
    </div>
  );
}

// ─── Code Viewer ─────────────────────────────────────────────────────
function CodeViewer() {
  const lines = [
    [1, false, <><span className="kw">use</span> <span className="va">std</span><span className="op">::</span><span className="ty">sync</span><span className="op">::</span><span className="ty">Arc</span>;</>],
    [2, false, <><span className="kw">use</span> <span className="va">gpui</span><span className="op">::</span>{"{"}<span className="ty">Entity</span>, <span className="ty">Context</span>, <span className="ty">Render</span>{"}"};</>],
    [3, false, ""],
    [4, false, <span className="co">/// Binary tree of split panes (V3-003 MS-1).</span>],
    [5, false, <><span className="kw">pub struct</span> <span className="ty">PaneTree</span> {"{"}</>],
    [6, false, <>    <span className="va">root</span><span className="op">:</span> <span className="ty">PaneNode</span>,</>],
    [7, false, <>    <span className="va">focused</span><span className="op">:</span> <span className="ty">Option</span><span className="op">&lt;</span><span className="ty">PaneId</span><span className="op">&gt;</span>,</>],
    [8, false, <>{"}"}</>],
    [9, false, ""],
    [10, false, <><span className="kw">impl</span> <span className="ty">PaneTree</span> {"{"}</>],
    [11, false, <>    <span className="kw">pub fn</span> <span className="fn">split</span>(<span className="op">&</span><span className="kw">mut</span> <span className="va">self</span>, <span className="va">dir</span><span className="op">:</span> <span className="ty">Dir</span>) <span className="op">-&gt;</span> <span className="ty">Result</span><span className="op">&lt;</span><span className="ty">PaneId</span><span className="op">&gt;</span> {"{"}</>],
    [12, false, <>        <span className="kw">let</span> <span className="va">id</span> <span className="op">=</span> <span className="ty">PaneId</span><span className="op">::</span><span className="fn">new</span>();</>],
    [13, true,  <>        <span className="va">self</span>.root.<span className="fn">insert_unwrap</span>(<span className="va">id</span>);  <span className="co">// no min-size check</span></>],
    [14, false, <>        <span className="kw">if</span> <span className="va">self</span>.<span className="fn">size_of</span>(<span className="va">id</span>) <span className="op">&lt;</span> <span className="nu">240</span> {"{"}</>],
    [15, false, <>            <span className="kw">return</span> <span className="ty">Err</span>(<span className="ty">SplitError</span><span className="op">::</span><span className="ty">TooSmall</span>);</>],
    [16, false, <>        {"}"}</>],
    [17, false, <>        <span className="ty">Ok</span>(<span className="va">id</span>)</>],
    [18, false, <>    {"}"}</>],
    [19, false, <>{"}"}</>],
  ];
  return (
    <div className="code">
      <div className="gut">
        {lines.map(([n, diag]) => (
          <span key={n} className={"ln" + (diag ? " diag" : "")}>{n}</span>
        ))}
      </div>
      <div className="src">
        {lines.map(([n, diag, content]) => (
          <div key={n} className={diag ? "diag-bg" : ""}>{content || "\u00a0"}</div>
        ))}
        <div style={{ marginTop: 8, padding: "8px 12px", background: "rgba(196,123,42,0.12)",
          borderLeft: "3px solid #c47b2a", borderRadius: 4, fontSize: 11, color: "var(--fg-2)",
          fontFamily: "var(--font-sans)", whiteSpace: "normal" }}>
          <strong style={{ color: "#c47b2a" }}>warn[L13]</strong> · rust-analyzer · prefer <code>insert_checked</code> when min-size matters · <span style={{ color: "var(--fg-3)" }}>Quick fix → ⌘.</span>
        </div>
      </div>
    </div>
  );
}

// ─── Markdown Viewer ─────────────────────────────────────────────────
function MarkdownViewer() {
  return (
    <div className="md-frame">
      <div className="md-gutter">
        <div className="gtag">@SPEC<br/>V3-006</div>
        <div className="gtag">@DOC<br/>MS-1</div>
        <div className="gtag">@TEST<br/>L42</div>
        <div className="gtag">@CODE<br/>L84</div>
      </div>
      <div className="md-scroll">
        <div className="md">
          <h1>SPEC-V3-006 · Markdown Viewer</h1>
          <p>
            CommonMark + GFM 렌더러로, <strong>@MX 태그</strong>를 좌측 거터에 자동 정렬한다.
            tree-sitter 기반 syntax highlight, 1,000+ 라인 가상 스크롤.
          </p>
          <div style={{ display:"flex", gap:6, marginBottom: 18 }}>
            <span className="mx-tag">@SPEC:V3-006</span>
            <span className="mx-tag">@DOC:MS-1</span>
            <span className="mx-tag">@CODE:src/md/render.rs</span>
          </div>
          <h2>Acceptance criteria</h2>
          <ul>
            <li>CommonMark 0.31 conformance ≥ 99% (tested via spec suite)</li>
            <li>@MX gutter alignment within ±2px across all heading levels</li>
            <li>Cold render of 10 KB doc &lt; 80ms on M2 baseline</li>
          </ul>
          <h3>Code blocks</h3>
          <pre>{`fn `}<span className="kw">parse</span>{`(src: &str) -> `}<span className="kw">Result</span>{`<Ast> {
    `}<span className="co">// tree-sitter handles fences + tables</span>{`
    Ast::`}<span className="kw">build</span>{`(src)
}`}</pre>
          <blockquote>
            <strong>Note ·</strong> @MX 태그가 라인보다 짧으면 가장 가까운 가시 라인에 스냅된다.
          </blockquote>
          <h3>Token reference</h3>
          <table>
            <thead><tr><th>Token</th><th>Role</th><th>Value</th></tr></thead>
            <tbody>
              <tr><td><code>md.body</code></td><td>본문</td><td>16/1.75 Pretendard</td></tr>
              <tr><td><code>md.code</code></td><td>코드</td><td>JetBrains Mono 12.5</td></tr>
              <tr><td><code>md.h1</code></td><td>제목</td><td>30/1.15 -0.04em</td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

// ─── Agent Dashboard ─────────────────────────────────────────────────
function AgentDashboard() {
  return (
    <div className="ag">
      <div className="ag-head">
        <div>
          <div className="ag-title">SPEC-V3-006 · MS-1 implementation</div>
          <div className="ag-sub">Claude Opus 4.7 · session 0a83e1 · started 2 min ago</div>
        </div>
        <div className="ag-spend">
          <div className="ag-stat"><span className="v">$0.042</span><span className="l">session</span></div>
          <div className="ag-stat"><span className="v">$1.23</span><span className="l">today</span></div>
          <div className="ag-stat"><span className="v" style={{ color: "var(--mint)" }}>14/27</span><span className="l">events</span></div>
          <div className="ag-stat"><span className="v">3:42</span><span className="l">runtime</span></div>
        </div>
      </div>
      <div className="ag-grid">
        <div className="ag-col">
          <div className="ag-colhd">Filters</div>
          <div className="ag-filters">
            <div className="ag-chip on"><span className="sw" style={{ background: "var(--accent)" }}/>session<span className="cnt">2</span></div>
            <div className="ag-chip on"><span className="sw" style={{ background: "var(--violet)" }}/>tool_use<span className="cnt">6</span></div>
            <div className="ag-chip on"><span className="sw" style={{ background: "var(--mint)" }}/>result<span className="cnt">5</span></div>
            <div className="ag-chip on"><span className="sw" style={{ background: "var(--accent)" }}/>message<span className="cnt">9</span></div>
            <div className="ag-chip"><span className="sw" style={{ background: "var(--amber)" }}/>hook<span className="cnt">3</span></div>
            <div className="ag-chip"><span className="sw" style={{ background: "var(--crimson)" }}/>error<span className="cnt">1</span></div>
            <div style={{ marginTop: 12, padding: "10px 0 0", borderTop: "1px solid var(--border)", fontSize: 10, color: "var(--fg-3)", textTransform: "uppercase", letterSpacing: "0.08em", fontWeight: 600 }}>Tools</div>
            <div className="ag-chip on" style={{ marginTop: 6 }}><span className="sw" style={{ background: "#4f9fce" }}/>Bash<span className="cnt">3</span></div>
            <div className="ag-chip on"><span className="sw" style={{ background: "#88b780" }}/>Read<span className="cnt">2</span></div>
            <div className="ag-chip on"><span className="sw" style={{ background: "#c792ea" }}/>Edit<span className="cnt">1</span></div>
          </div>
        </div>
        <div className="ag-col">
          <div className="ag-colhd" style={{ display:"flex", justifyContent:"space-between" }}>
            <span>Event timeline · 27 events</span>
            <span style={{ color: "var(--mint)", textTransform:"none", letterSpacing:0, fontWeight: 500 }}>● live</span>
          </div>
          <div className="ag-tl">
            <div className="ag-ev msg"><span className="ts">13:45:30</span><I.zap size={14}/><div className="body"><div className="name">session_start</div><div className="meta">Opus 4.7 · context 89.2 KB</div></div><span className="dur">0.02s</span></div>
            <div className="ag-ev msg"><span className="ts">13:45:31</span><I.msg size={14}/><div className="body"><div className="name">message_start</div><div className="meta">user · 412 tok in</div></div><span className="dur">0.21s</span></div>
            <div className="ag-ev msg"><span className="ts">13:45:33</span><I.msg size={14}/><div className="body"><div className="name">message_delta</div><div className="meta">streaming · 1,204 tok</div></div><span className="dur">2.4s</span></div>
            <div className="ag-ev tool expanded"><span className="ts">13:45:36</span><I.wrench size={14}/><div className="body"><div className="name">tool_use · Bash</div><div className="meta"><code>cargo test --workspace</code></div>
              <div className="detail">{`{
  "tool": "Bash",
  "input": "cargo test --workspace -p moai-md",
  "exit_code": 0,
  "duration_ms": 4217,
  "stdout_lines": 286
}`}</div>
            </div><span className="dur">4.21s</span></div>
            <div className="ag-ev ok"><span className="ts">13:45:40</span><I.check size={14}/><div className="body"><div className="name">tool_use_result</div><div className="meta">17 passed · 1 failed</div></div><span className="dur">0.01s</span></div>
            <div className="ag-ev tool"><span className="ts">13:45:41</span><I.wrench size={14}/><div className="body"><div className="name">tool_use · Read</div><div className="meta"><code>src/panes/persist.rs</code></div></div><span className="dur">0.08s</span></div>
            <div className="ag-ev tool"><span className="ts">13:45:42</span><I.wrench size={14}/><div className="body"><div className="name">tool_use · Edit</div><div className="meta">3 hunks · L142, L167, L201</div></div><span className="dur">0.12s</span></div>
            <div className="ag-ev msg"><span className="ts">13:45:44</span><I.msg size={14}/><div className="body"><div className="name">message_delta</div><div className="meta">2,830 tok · thinking</div></div><span className="dur">3.1s</span></div>
            <div className="ag-ev err"><span className="ts">13:45:48</span><I.alertTri size={14}/><div className="body"><div className="name">hook · pre_commit</div><div className="meta" style={{ color: "var(--crimson)" }}>format check failed</div></div><span className="dur">0.4s</span></div>
            <div className="ag-ev tool"><span className="ts">13:45:49</span><I.wrench size={14}/><div className="body"><div className="name">tool_use · Bash</div><div className="meta"><code>cargo fmt</code></div></div><span className="dur">0.6s</span></div>
            <div className="ag-ev ok"><span className="ts">13:45:50</span><I.check size={14}/><div className="body"><div className="name">tool_use_result</div><div className="meta">7 files reformatted</div></div><span className="dur">0.01s</span></div>
          </div>
        </div>
        <div className="ag-col">
          <div className="ag-colhd">Insights</div>
          <div className="ag-side">
            <div className="ag-card">
              <h4>Cost · 7 days</h4>
              <div className="ag-bar">
                {[0.4,0.7,0.5,1.1,0.8,1.4,1.23].map((v,i) => (
                  <div key={i} className={"b" + (i === 6 ? " cur" : "")} style={{ height: `${v*40}px` }}/>
                ))}
              </div>
              <div className="ag-bar-x">{["M","T","W","T","F","S","S"].map((d,i)=>(<span key={i}>{d}</span>))}</div>
            </div>
            <div className="ag-card">
              <h4>Instructions · 89.2 KB</h4>
              <div className="ag-tree">
                <div className="l1">CLAUDE.md <span className="sz">62.4 KB</span></div>
                <div className="l2">moai-constitution.md <span className="sz">8.1 KB</span></div>
                <div className="l2">agent-protocol.md <span className="sz">12.8 KB</span></div>
                <div className="l1">Skills <span className="sz">16.8 KB</span></div>
                <div className="l2">moai-foundation-core <span className="sz">12.4 KB</span></div>
                <div className="l2">workflow-project <span className="sz">4.4 KB</span></div>
                <div className="l1">Memory <span className="sz">10.0 KB</span></div>
                <div className="l3">user_profile.md</div>
                <div className="l3">lessons.md</div>
              </div>
            </div>
            <div className="ag-card">
              <h4>Control</h4>
              <div className="ag-ctrl">
                <button className="ag-btn"><I.pause size={12}/> Pause</button>
                <button className="ag-btn primary"><I.play size={12}/> Continue</button>
                <button className="ag-btn danger"><I.stop size={12}/> Kill</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ─── Git Management ──────────────────────────────────────────────────
function GitMgmt() {
  return (
    <div style={{ display: "grid", gridTemplateColumns: "320px 1fr", height: "100%" }}>
      <div className="git" style={{ borderRight: "1px solid var(--border)" }}>
        <h3>Branch</h3>
        <div className="branch-row cur"><I.branch size={14}/><span className="nm">feature/v3-006-md</span><span className="ahead">↑3 ↓1</span></div>
        <div className="branch-row"><I.branch size={14}/><span className="nm">main</span><span className="ahead">→ origin</span></div>
        <div className="branch-row"><I.branch size={14}/><span className="nm">feature/v3-005</span><span className="ahead">↑12</span></div>

        <h3>Staged · 2</h3>
        <div className="row"><span className="ck on"/><span className="st A">A</span><span className="pa">src/md/<b>render.rs</b></span><span className="da">+184</span></div>
        <div className="row"><span className="ck on"/><span className="st M">M</span><span className="pa">src/md/<b>parser.rs</b></span><span className="da">+24 −7</span></div>

        <h3>Changes · 4</h3>
        <div className="row"><span className="ck"/><span className="st M">M</span><span className="pa">src/panes/<b>persist.rs</b></span><span className="da">+12 −3</span></div>
        <div className="row"><span className="ck"/><span className="st M">M</span><span className="pa">Cargo.toml</span><span className="da">+1</span></div>
        <div className="row"><span className="ck"/><span className="st A">A</span><span className="pa">tests/<b>md_render.rs</b></span><span className="da">+89</span></div>
        <div className="row"><span className="ck"/><span className="st Q">?</span><span className="pa">.moai/cache/build.log</span><span className="da"></span></div>

        <div className="commitbox">
          <input placeholder="feat(md): @MX gutter alignment within ±2px"/>
          <textarea rows="3" placeholder="Detailed message — Cmd+Enter to commit" defaultValue="Implements SPEC-V3-006 MS-1 acceptance criteria.

- Tree-sitter render path for CommonMark 0.31
- @MX gutter alignment within ±2px across heading levels"/>
          <div className="actions">
            <span className="check"><I.check size={12}/> Sign-off</span>
            <span className="check"><I.check size={12}/> moai pre-commit ✓</span>
            <button className="commit-btn">Commit · ⌘⏎</button>
          </div>
        </div>
      </div>
      <div style={{ display:"flex", flexDirection: "column", minWidth:0 }}>
        <div style={{ height: 38, borderBottom: "1px solid var(--border)", display:"flex", alignItems:"center", padding: "0 16px", gap: 12, fontSize: 12, color: "var(--fg-2)", background: "var(--panel)" }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: 11.5, color: "var(--fg)" }}>src/md/render.rs</span>
          <span style={{ color: "var(--mint)" }}>+184</span>
          <span style={{ color: "var(--crimson)" }}>−0</span>
          <span style={{ marginLeft: "auto", display:"flex", gap: 8, fontSize: 11 }}>
            <span style={{ padding: "2px 8px", border: "1px solid var(--border-strong)", borderRadius: 4, color: "var(--fg-2)" }}>Unified</span>
            <span style={{ padding: "2px 8px", border: "1px solid var(--accent)", color: "var(--accent)", borderRadius: 4, fontWeight: 600 }}>Split</span>
          </span>
        </div>
        <div className="gitdiff">
          <div className="hunk">@@ -0,0 +1,42 @@ src/md/render.rs</div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">1</span><span className="sym">+</span><span>use tree_sitter::{`{Parser, Tree}`};</span></div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">2</span><span className="sym">+</span><span>use crate::ast::{`{Ast, Node}`};</span></div>
          <div className="ln"><span className="nrA"></span><span className="nrB">3</span><span className="sym"></span><span> </span></div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">4</span><span className="sym">+</span><span>/// Render a CommonMark string into AST + @MX gutter map.</span></div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">5</span><span className="sym">+</span><span>pub fn render(src: &str) -&gt; Result&lt;RenderedDoc&gt; {`{`}</span></div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">6</span><span className="sym">+</span><span>    let mut parser = Parser::new();</span></div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">7</span><span className="sym">+</span><span>    parser.set_language(tree_sitter_md::language())?;</span></div>
          <div className="hunk">@@ -42,8 +56,12 @@ fn align_gutter()</div>
          <div className="ln"><span className="nrA">42</span><span className="nrB">56</span><span className="sym"></span><span>    for tag in tags {`{`}</span></div>
          <div className="ln del"><span className="nrA">43</span><span className="nrB"></span><span className="sym">−</span><span>        let y = tag.line * line_height;</span></div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">57</span><span className="sym">+</span><span>        let y = snap_to_visible_line(tag.line, line_height);</span></div>
          <div className="ln add"><span className="nrA"></span><span className="nrB">58</span><span className="sym">+</span><span>        debug_assert!(y % line_height &lt;= 2.0);</span></div>
          <div className="ln"><span className="nrA">44</span><span className="nrB">59</span><span className="sym"></span><span>        gutter.push(GutterEntry {`{ tag, y }`});</span></div>
          <div className="ln"><span className="nrA">45</span><span className="nrB">60</span><span className="sym"></span><span>    {`}`}</span></div>
        </div>
      </div>
    </div>
  );
}

// ─── SPEC Management (Kanban) ────────────────────────────────────────
function SpecKanban() {
  const card = (id, title, tags, ac, who) => (
    <div className="card">
      <div className="id">{id}</div>
      <div className="ti">{title}</div>
      <div style={{ marginTop: 6 }}>{tags.map((t,i) => <span key={i} className="tag">{t}</span>)}</div>
      <div className="meta">
        <span className="ac">
          {ac.map((c,i) => <span key={i} className={"pip " + c}/>)}
        </span>
        <span style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 5 }}>
          <span className="av">{who}</span>
        </span>
      </div>
    </div>
  );
  return (
    <div className="spec">
      <div className="kanban">
        <div className="col draft">
          <h4>Draft <span className="badge">2</span></h4>
          <div className="stack">
            {card("V3-011", "Search · global ripgrep + LSP symbols", ["@SPEC","@plan"], ["","",""], "JS")}
            {card("V3-012", "Settings · keybind editor", ["@SPEC"], ["","",""], "MK")}
          </div>
        </div>
        <div className="col plan">
          <h4>Planned <span className="badge">3</span></h4>
          <div className="stack">
            {card("V3-008", "Git Management · status, diff, commit", ["@SPEC","@plan","@design"], ["g","g",""], "MK")}
            {card("V3-009", "SPEC Management · kanban + AC", ["@SPEC","@design"], ["g","",""], "JS")}
            {card("V3-010", "Agent Dashboard · timeline + cost", ["@SPEC","@design"], ["g","g","y"], "JS")}
          </div>
        </div>
        <div className="col dev">
          <h4>In&nbsp;dev <span className="badge">2</span></h4>
          <div className="stack">
            {card("V3-006", "Markdown Viewer · CommonMark + @MX gutter", ["@SPEC","@code","@test"], ["g","g","g","y"], "JS")}
            {card("V3-007", "Code Viewer · LSP + tree-sitter", ["@SPEC","@code"], ["g","y","r"], "MK")}
          </div>
        </div>
        <div className="col done">
          <h4>Done <span className="badge">3</span></h4>
          <div className="stack">
            {card("V3-005", "File Explorer · git status + fuzzy", ["@DONE"], ["g","g","g"], "JS")}
            {card("V3-003", "Panes & Tabs · binary tree", ["@DONE","@locked"], ["g","g","g","g"], "MK")}
            {card("V3-002", "Terminal · libghostty 60fps", ["@DONE","@locked"], ["g","g","g","g","g"], "MK")}
          </div>
        </div>
      </div>
    </div>
  );
}

// ─── Web Browser ─────────────────────────────────────────────────────
function WebBrowser() {
  return (
    <div style={{ display: "flex", flexDirection:"column", height: "100%", background: "var(--bg)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "8px 12px", borderBottom: "1px solid var(--border)", background: "var(--panel)" }}>
        <button style={{ background: "transparent", border: "none", color: "var(--fg-2)", cursor:"pointer" }}>‹</button>
        <button style={{ background: "transparent", border: "none", color: "var(--fg-3)", cursor:"pointer" }}>›</button>
        <button style={{ background: "transparent", border: "none", color: "var(--fg-2)", cursor:"pointer" }}><I.refresh size={13}/></button>
        <div style={{ flex: 1, height: 28, background: "var(--surface)", border: "1px solid var(--border)", borderRadius: 14, display:"flex", alignItems:"center", padding: "0 12px", gap: 6, fontFamily:"var(--font-mono)", fontSize: 11.5, color: "var(--fg)" }}>
          <span style={{ color: "var(--mint)" }}>●</span>
          <span style={{ color: "var(--fg-3)" }}>localhost:5173</span>
          <span>/projects/v3-006</span>
          <span style={{ marginLeft: "auto", padding: "1px 6px", background: "var(--accent-soft)", color: "var(--accent)", borderRadius: 3, fontSize: 9.5 }}>DEV</span>
        </div>
        <button style={{ background: "transparent", border: "none", color: "var(--fg-2)", cursor:"pointer" }}><I.settings size={13}/></button>
      </div>
      <div style={{ flex: 1, padding: 24, overflow: "auto", background: "var(--bg)" }}>
        <div style={{ maxWidth: 540, margin: "0 auto", color: "var(--fg)" }}>
          <div style={{ height: 8, width: 60, background: "var(--accent-soft)", borderRadius: 999, marginBottom: 16 }}/>
          <div style={{ fontSize: 28, fontWeight: 800, letterSpacing: "-0.04em", marginBottom: 8, color: "var(--fg)" }}>Markdown preview</div>
          <div style={{ fontSize: 13, color: "var(--fg-3)", marginBottom: 24 }}>Live HMR · 184 ms last reload</div>
          <div style={{ height: 240, background: "var(--surface)", border: "1px solid var(--border)", borderRadius: 12, display: "grid", placeItems: "center", color: "var(--fg-3)", fontSize: 12 }}>
            <div style={{ textAlign: "center" }}>
              <I.globe size={36}/>
              <div style={{ marginTop: 10 }}>Iframe preview · render output</div>
            </div>
          </div>
        </div>
      </div>
      <div style={{ height: 28, borderTop: "1px solid var(--border)", display:"flex", alignItems:"center", padding: "0 12px", gap: 14, fontFamily:"var(--font-mono)", fontSize: 10, color: "var(--fg-3)" }}>
        <span>200 OK</span><span>184 ms</span><span style={{ color: "var(--mint)" }}>● HMR</span><span style={{ marginLeft:"auto" }}>F12 · DevTools</span>
      </div>
    </div>
  );
}

Object.assign(window, {
  FileTree, FileRow, SAMPLE_TREE,
  Terminal, CodeViewer, MarkdownViewer,
  AgentDashboard, GitMgmt, SpecKanban, WebBrowser,
});
