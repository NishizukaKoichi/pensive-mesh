import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import "./styles.css";

type View =
  "today" | "sources" | "inbox" | "ask" | "context" | "recovery" | "audit";

interface VaultStatus {
  vault_id: string;
  locked: boolean;
  frozen: boolean;
  frozen_reason: string | null;
  recovery_exported: boolean;
  last_recovery_test: string | null;
  source_count: number;
  fragment_count: number;
  memory_inbox_count: number;
  accepted_memory_count: number;
  conflict_count: number;
  context_pack_count: number;
  external_models_enabled: boolean;
  network_activity: string;
}

interface SourceSummary {
  source_id: string;
  source_type: string;
  provider: string;
  external_id: string | null;
  title: string | null;
  captured_at: string;
  occurred_from: string | null;
  occurred_to: string | null;
  sensitivity: string;
  state: string;
  fragment_count: number;
  content_object_id: string;
  integrity: string;
}

interface FragmentSummary {
  fragment_id: string;
  source_id: string;
  external_id: string | null;
  parent_external_id: string | null;
  role: string | null;
  occurred_at: string | null;
  locator: Record<string, unknown>;
  text: string;
  sensitivity: string;
  secret_candidate: boolean;
  injection_candidate: boolean;
}

interface MemorySummary {
  memory_id: string;
  memory_type: string;
  statement: string;
  epistemic_status: string;
  review_state: string;
  evidence_strength: string;
  valid_from: string | null;
  valid_to: string | null;
  sensitivity: string;
  third_party: boolean;
  current_revision: number;
  evidence: FragmentSummary[];
  created_at: string;
  reviewed_at: string | null;
}

interface SearchHit {
  fragment: FragmentSummary;
  source_title: string | null;
  source_provider: string;
  source_state: string;
  rank: number;
  accepted_memories: string[];
  contradictions: string[];
  why_used: string;
}

interface ContextPack {
  protocol: string;
  pack_id: string;
  purpose: string;
  query: string;
  created_at: string;
  expires_at: string;
  summary: string;
  memory_items: MemorySummary[];
  contradictions: string[];
  source_fragments: FragmentSummary[];
  omissions: string[];
  redactions: string[];
  integrity: {
    canonical_digest: string;
    signed_by_device: string;
    signature: string;
  };
}

const isTauri = "__TAURI_INTERNALS__" in window;
const app = document.querySelector<HTMLDivElement>("#app") as HTMLDivElement;
if (!app) throw new Error("Application mount point is missing");

const state: {
  view: View;
  status: VaultStatus | null;
  sources: SourceSummary[];
  fragments: FragmentSummary[];
  memories: MemorySummary[];
  hits: SearchHit[];
  contextPack: ContextPack | null;
  selectedSource: SourceSummary | null;
  busy: boolean;
  toast: string | null;
  error: string | null;
  onboarding: boolean;
} = {
  view: "today",
  status: null,
  sources: [],
  fragments: [],
  memories: [],
  hits: [],
  contextPack: null,
  selectedSource: null,
  busy: false,
  toast: null,
  error: null,
  onboarding: false,
};

const demoFragments: FragmentSummary[] = [
  {
    fragment_id: "fragment-demo-01",
    source_id: "source-demo-01",
    external_id: "message-root",
    parent_external_id: null,
    role: "user",
    occurred_at: "2026-07-26T08:42:00.000Z",
    locator: { kind: "chatgpt_message", branch_path: ["message-root"] },
    text: "原資料は消さず、推論は疑い、矛盾は残す。",
    sensitivity: "PERSONAL",
    secret_candidate: false,
    injection_candidate: false,
  },
  {
    fragment_id: "fragment-demo-02",
    source_id: "source-demo-01",
    external_id: "message-branch-a",
    parent_external_id: "message-root",
    role: "assistant",
    occurred_at: "2026-07-26T08:43:00.000Z",
    locator: {
      kind: "chatgpt_message",
      branch_path: ["message-root", "message-branch-a"],
    },
    text: "自己理解は、証拠と反証を持つ仮説として保存する。",
    sensitivity: "HIGHLY_SENSITIVE",
    secret_candidate: false,
    injection_candidate: false,
  },
];

function installDemo(): void {
  state.status = {
    vault_id: "sample-vault-7f2a",
    locked: false,
    frozen: false,
    frozen_reason: null,
    recovery_exported: false,
    last_recovery_test: null,
    source_count: 12,
    fragment_count: 384,
    memory_inbox_count: 3,
    accepted_memory_count: 28,
    conflict_count: 1,
    context_pack_count: 4,
    external_models_enabled: false,
    network_activity: "OFFLINE_ONLY",
  };
  state.sources = [
    {
      source_id: "source-demo-01",
      source_type: "conversation_export",
      provider: "chatgpt",
      external_id: "conversation-demo",
      title: "Pensive の安全境界",
      captured_at: "2026-07-26T08:45:00.000Z",
      occurred_from: "2026-07-26T08:42:00.000Z",
      occurred_to: "2026-07-26T08:44:00.000Z",
      sensitivity: "SENSITIVE",
      state: "ACTIVE",
      fragment_count: 42,
      content_object_id: "6a2e8f460d9a8b3d",
      integrity: "blake3:6a2e8f460d9a8b3d",
    },
    {
      source_id: "source-demo-02",
      source_type: "file",
      provider: "local",
      external_id: "ritual-notes.md",
      title: "Ritual の観察メモ",
      captured_at: "2026-07-24T11:10:00.000Z",
      occurred_from: null,
      occurred_to: null,
      sensitivity: "PERSONAL",
      state: "ACTIVE",
      fragment_count: 8,
      content_object_id: "178a1c0df07d9e33",
      integrity: "blake3:178a1c0df07d9e33",
    },
  ];
  state.fragments = demoFragments;
  state.selectedSource = state.sources[0] ?? null;
  state.memories = [
    {
      memory_id: "memory-demo-01",
      memory_type: "DECISION",
      statement: "Pensive は本人の承認なしに外部作用を行わない。",
      epistemic_status: "INFERRED",
      review_state: "CANDIDATE",
      evidence_strength: "HIGH",
      valid_from: null,
      valid_to: null,
      sensitivity: "PERSONAL",
      third_party: false,
      current_revision: 1,
      evidence: [demoFragments[0]!],
      created_at: "2026-07-26T08:46:00.000Z",
      reviewed_at: null,
    },
    {
      memory_id: "memory-demo-02",
      memory_type: "SELF_MODEL",
      statement:
        "現場観察型のコンテキスト設計者、という仮説が複数回現れている。",
      epistemic_status: "INFERRED",
      review_state: "CANDIDATE",
      evidence_strength: "MEDIUM",
      valid_from: "2026-01-01T00:00:00.000Z",
      valid_to: null,
      sensitivity: "HIGHLY_SENSITIVE",
      third_party: false,
      current_revision: 1,
      evidence: [demoFragments[1]!],
      created_at: "2026-07-26T08:47:00.000Z",
      reviewed_at: null,
    },
  ];
}

async function start(): Promise<void> {
  if (!isTauri) {
    installDemo();
    render();
    return;
  }
  try {
    state.status = await invoke<VaultStatus | null>("vault_status");
    state.onboarding = state.status === null;
    if (state.status) await refreshAll();
  } catch (error) {
    state.error = messageFrom(error);
    state.onboarding = true;
  }
  render();
}

async function refreshAll(): Promise<void> {
  if (!isTauri || !state.status) return;
  const [status, sources, memories] = await Promise.all([
    invoke<VaultStatus | null>("vault_status"),
    invoke<SourceSummary[]>("list_sources"),
    invoke<MemorySummary[]>("memory_inbox"),
  ]);
  state.status = status;
  state.sources = sources;
  state.memories = memories;
  if (state.selectedSource) {
    const replacement = sources.find(
      (source) => source.source_id === state.selectedSource?.source_id,
    );
    state.selectedSource = replacement ?? null;
  }
}

function render(): void {
  app.innerHTML = `${shell()}${state.onboarding ? onboarding() : ""}${toast()}`;
  wireEvents();
}

function shell(): string {
  const status = state.status;
  const locked = status === null;
  return `
    <div class="app-shell ${locked ? "is-locked" : ""}">
      <aside class="navigation" aria-label="主な画面">
        <div class="brand-row">
          <span class="brand-mark" aria-hidden="true"><i></i></span>
          <div><strong>Pensive</strong><span>Mesh / v0.1</span></div>
        </div>
        <nav>${navItems()}</nav>
        <div class="nav-foot">
          <div class="connection-state"><i></i><span>外部通信なし</span></div>
          <button class="quiet-button" data-action="lock" ${locked ? "disabled" : ""}>保管庫をロック</button>
        </div>
      </aside>
      <main id="workspace" class="workspace" tabindex="-1">
        ${header()}
        <div class="view" data-view="${state.view}">${viewContent()}</div>
      </main>
      ${inspector()}
    </div>`;
}

function navItems(): string {
  const items: Array<[View, string, string]> = [
    ["today", "今日", "⌁"],
    ["sources", "原資料", "◫"],
    ["inbox", "記憶の確認", "◇"],
    ["ask", "Pensiveに訊く", "⌕"],
    ["context", "文脈パック", "◈"],
    ["recovery", "保存と復旧", "↺"],
    ["audit", "監査", "◎"],
  ];
  return items
    .map(
      ([view, label, glyph]) =>
        `<button class="nav-item ${state.view === view ? "active" : ""}" data-view="${view}" aria-current="${state.view === view ? "page" : "false"}"><span aria-hidden="true">${glyph}</span>${label}</button>`,
    )
    .join("");
}

function header(): string {
  const labels: Record<View, [string, string]> = {
    today: ["今日", "保管庫の現在地と、確認が必要なこと。"],
    sources: ["原資料", "残されていたものを、改変せずに辿る。"],
    inbox: ["記憶の確認", "AIの候補を、本人の真実にする前に確かめる。"],
    ask: ["Pensiveに訊く", "答えより先に、根拠と時点を選ぶ。"],
    context: ["文脈パック", "目的に必要な記憶だけを、期限付きで束ねる。"],
    recovery: ["保存と復旧", "安全は、復旧できると確かめて初めて成立する。"],
    audit: ["監査", "本文を残さず、意味変更の連鎖を検証する。"],
  };
  const [title, subtitle] = labels[state.view];
  return `<header class="view-header"><div><p>${subtitle}</p><h1>${title}</h1></div><div class="header-actions">${!isTauri ? '<span class="preview-label">サンプル表示</span>' : ""}<button class="primary-button" data-action="import" ${state.status ? "" : "disabled"}>原資料を取り込む</button></div></header>`;
}

function viewContent(): string {
  if (!state.status) return lockedView();
  switch (state.view) {
    case "today":
      return todayView();
    case "sources":
      return sourcesView();
    case "inbox":
      return inboxView();
    case "ask":
      return askView();
    case "context":
      return contextView();
    case "recovery":
      return recoveryView();
    case "audit":
      return auditView();
  }
}

function lockedView(): string {
  return `<section class="empty-state"><span class="empty-orbit" aria-hidden="true"></span><p>保管庫はロックされています</p><h2>鍵を開くまで、記憶はここに現れません。</h2><button class="primary-button" data-action="show-onboarding">保管庫を開く</button></section>`;
}

function todayView(): string {
  const status = state.status!;
  return `
    <section class="status-line" aria-label="保管庫の主要状態">
      ${metric(status.source_count, "原資料", "暗号化して保存")}
      ${metric(status.memory_inbox_count, "確認待ち", status.memory_inbox_count ? "本人の判断が必要" : "すべて確認済み")}
      ${metric(status.conflict_count, "未解決の矛盾", status.conflict_count ? "消さずに保持中" : "現在はなし")}
      ${metric(status.context_pack_count, "文脈パック", "すべて期限付き")}
    </section>
    <section class="today-grid">
      <div class="working-list">
        <div class="section-heading"><div><span>次に確かめる</span><h2>本人の判断を待っているもの</h2></div><button class="text-button" data-view="inbox">確認画面へ</button></div>
        ${state.memories.length ? state.memories.slice(0, 3).map(memoryRow).join("") : emptyRow("確認待ちの記憶はありません。")}
      </div>
      <div class="continuity-panel">
        <span class="eyebrow">CONTINUITY</span>
        <h2>${status.recovery_exported ? "復旧鍵は書き出し済み" : "復旧鍵がまだ外にありません"}</h2>
        <p>${status.recovery_exported ? `最終復旧試験: ${status.last_recovery_test ? dateLabel(status.last_recovery_test) : "未実施"}` : "この端末を失うと、運営者も記憶を戻せません。暗号化された Recovery Kit を別媒体へ保存してください。"}</p>
        <div class="continuity-track"><i class="${status.recovery_exported ? "done" : ""}"></i><i class="${status.last_recovery_test ? "done" : ""}"></i><i></i></div>
        <button class="secondary-button" data-view="recovery">保存と復旧を確認</button>
      </div>
    </section>
    <section class="recent-section">
      <div class="section-heading"><div><span>最近の原資料</span><h2>証拠の入口</h2></div><button class="text-button" data-view="sources">すべて見る</button></div>
      ${state.sources.length ? state.sources.slice(0, 4).map(sourceRow).join("") : emptyRow("原資料はまだありません。")}
    </section>`;
}

function metric(value: number, label: string, note: string): string {
  return `<div class="metric"><strong>${value.toLocaleString("ja-JP")}</strong><span>${label}</span><small>${note}</small></div>`;
}

function sourcesView(): string {
  return `
    <div class="toolbar"><label class="search-control"><span>⌕</span><input id="source-filter" type="search" placeholder="タイトル、由来、時期で絞る" aria-label="原資料を絞り込む" /></label><select id="source-sensitivity" aria-label="機密区分"><option value="all">すべての機密区分</option><option>PERSONAL</option><option>SENSITIVE</option><option>HIGHLY_SENSITIVE</option></select></div>
    <section class="source-table" aria-label="原資料一覧">
      <div class="table-head"><span>原資料</span><span>期間</span><span>機密</span><span>断片</span></div>
      <div id="source-rows">${state.sources.length ? state.sources.map(sourceRow).join("") : emptyRow("原資料を取り込むと、ここに出典と完全性が並びます。")}</div>
    </section>`;
}

function sourceRow(source: SourceSummary): string {
  return `<button class="source-row ${state.selectedSource?.source_id === source.source_id ? "selected" : ""}" data-source-id="${escapeAttr(source.source_id)}"><span><i class="source-glyph" aria-hidden="true">${source.source_type === "conversation_export" ? "◫" : "·"}</i><span><strong>${escapeHtml(source.title ?? "無題の原資料")}</strong><small>${escapeHtml(source.provider)} · ${escapeHtml(source.state)}</small></span></span><time>${dateLabel(source.occurred_from ?? source.captured_at)}</time><em class="sensitivity ${source.sensitivity.toLowerCase()}">${sensitivityLabel(source.sensitivity)}</em><b>${source.fragment_count.toLocaleString("ja-JP")}</b></button>`;
}

function inboxView(): string {
  return `<div class="inbox-intro"><span>${state.memories.length} 件</span><p>承認は「正しさの確定」ではなく、現時点で通常検索に使ってよいという本人の判断です。</p></div><section class="memory-list">${state.memories.length ? state.memories.map(memoryReview).join("") : emptyRow("確認待ちの候補はありません。")}</section>`;
}

function memoryRow(memory: MemorySummary): string {
  return `<button class="working-row" data-view="inbox"><span class="type-code">${escapeHtml(memory.memory_type)}</span><strong>${escapeHtml(memory.statement)}</strong><small>${memory.evidence.length}件の根拠 · ${sensitivityLabel(memory.sensitivity)}</small></button>`;
}

function memoryReview(memory: MemorySummary): string {
  const evidence = memory.evidence[0];
  return `<article class="memory-review"><div class="memory-meta"><span class="type-code">${escapeHtml(memory.memory_type)}</span><span>${escapeHtml(memory.epistemic_status)}</span><span>${sensitivityLabel(memory.sensitivity)}</span></div><h2>${escapeHtml(memory.statement)}</h2><blockquote><span>根拠 ${evidence ? escapeHtml(evidence.external_id ?? evidence.fragment_id.slice(0, 8)) : "なし"}</span>${evidence ? escapeHtml(evidence.text) : "根拠が失われています。承認できません。"}</blockquote><div class="review-actions"><button class="secondary-button" data-review="reject" data-memory-id="${escapeAttr(memory.memory_id)}">却下</button><button class="secondary-button" data-review="correct" data-memory-id="${escapeAttr(memory.memory_id)}">訂正</button><button class="primary-button" data-review="accept" data-memory-id="${escapeAttr(memory.memory_id)}" ${evidence ? "" : "disabled"}>記憶として使う</button></div></article>`;
}

function askView(): string {
  return `<section class="ask-composer"><label for="ask-query">問い</label><div><textarea id="ask-query" rows="3" placeholder="例：Pensiveの外部作用について、いつ何を決めた？">Pensiveの外部作用について何を決めた？</textarea><button class="primary-button" data-action="search">根拠を探す</button></div><div class="scope-controls"><label>時点 <select id="ask-time"><option>現在と履歴</option><option>現在のみ</option><option>過去の決定時点</option></select></label><label>機密 <select id="ask-sensitivity"><option>PERSONAL + SENSITIVE</option><option>PERSONALのみ</option></select></label><span>外部モデル: OFF</span></div></section>${state.hits.length ? `<section class="answer-area"><div class="answer-heading"><span>ローカル検索</span><h2>${state.hits.length}件の根拠が見つかりました</h2></div>${state.hits.map(hitRow).join("")}<button class="secondary-button" data-view="context">この範囲から文脈パックを作る</button></section>` : `<section class="ask-empty"><span aria-hidden="true">⌕</span><p>答えはまだ生成しません。</p><h2>まず、使うべき証拠と使わない情報を確認します。</h2></section>`}`;
}

function hitRow(hit: SearchHit): string {
  return `<article class="evidence-hit"><div><span>${escapeHtml(hit.source_provider)} · ${dateLabel(hit.fragment.occurred_at)}</span><strong>${escapeHtml(hit.source_title ?? "無題の原資料")}</strong></div><p>${escapeHtml(hit.fragment.text)}</p>${hit.contradictions.length ? `<aside>未解決: ${escapeHtml(hit.contradictions.join(" / "))}</aside>` : ""}<small>${escapeHtml(hit.why_used)}</small></article>`;
}

function contextView(): string {
  return `<section class="context-builder"><div><label for="context-purpose">目的</label><input id="context-purpose" value="Pensiveの安全境界を別のAIに説明する" /></div><div><label for="context-query">検索する文脈</label><input id="context-query" value="Pensive 外部作用 原資料 記憶" /></div><div class="context-options"><label>上限 <select id="context-budget"><option value="2000">Brief · 2,000</option><option value="8000" selected>Working · 8,000</option><option value="32000">Deep · 32,000</option></select></label><span>Secret 除外</span><span>第三者情報 除外</span></div><button class="primary-button" data-action="build-context">プレビューを作る</button></section>${state.contextPack ? contextPreview(state.contextPack) : `<section class="context-empty"><span class="empty-orbit small" aria-hidden="true"></span><h2>文脈パックは保管庫ではありません。</h2><p>特定の目的に必要な範囲だけを選び、24時間後に失効します。</p></section>`}`;
}

function contextPreview(pack: ContextPack): string {
  return `<section class="context-preview"><div class="preview-head"><div><span>${escapeHtml(pack.protocol)}</span><h2>${escapeHtml(pack.purpose)}</h2><p>${escapeHtml(pack.summary)}</p></div><button class="secondary-button" data-action="export-context" data-pack-id="${escapeAttr(pack.pack_id)}">暗号化して書き出す</button></div><dl><div><dt>根拠</dt><dd>${pack.source_fragments.length}</dd></div><div><dt>記憶</dt><dd>${pack.memory_items.length}</dd></div><div><dt>矛盾</dt><dd>${pack.contradictions.length}</dd></div><div><dt>除外</dt><dd>${pack.omissions.length}</dd></div></dl><div class="digest"><span>署名済み digest</span><code>${escapeHtml(pack.integrity.canonical_digest)}</code><small>有効期限 ${dateLabel(pack.expires_at)}</small></div>${pack.omissions.length ? `<div class="omissions"><strong>含めなかったもの</strong>${pack.omissions.map((item) => `<p>${escapeHtml(item)}</p>`).join("")}</div>` : ""}</section>`;
}

function recoveryView(): string {
  const status = state.status!;
  return `<section class="recovery-status"><div class="recovery-symbol ${status.recovery_exported ? "ready" : ""}" aria-hidden="true"><span></span></div><div><span class="eyebrow">RECOVERY READINESS</span><h2>${status.recovery_exported ? "Recovery Kit は書き出されています" : "この保管庫は、まだ一台の端末に依存しています"}</h2><p>${status.recovery_exported ? `最終 clean restore: ${status.last_recovery_test ? dateLabel(status.last_recovery_test) : "未検証"}` : "Recovery Kit と暗号化 Backup の両方がなければ、端末喪失後に復旧できません。"}</p></div></section><section class="recovery-actions"><article><span>01</span><div><h3>Recovery Kit を作る</h3><p>Vault Root Key を別の強いパスフレーズで暗号化します。</p></div><button class="secondary-button" data-action="export-recovery">書き出す</button></article><article><span>02</span><div><h3>暗号化 Backup を作る</h3><p>SQLCipher catalog、Source object、署名 Journal を整合した状態で複製します。</p></div><button class="secondary-button" data-action="backup">保存先を選ぶ</button></article><article><span>03</span><div><h3>Clean restore を試す</h3><p>別の一時環境で鍵・hash・audit chain を読み直します。</p></div><button class="secondary-button" data-action="recovery-test">復旧を検証</button></article></section><aside class="truth-note"><strong>正直な現在地</strong><p>v0.1 はローカル復旧を検証します。Arcane の地理分散復旧と複数物理端末は、実証されるまで安全表示しません。</p></aside>`;
}

function auditView(): string {
  const status = state.status!;
  return `<section class="integrity-hero"><div class="integrity-ring ${status.frozen ? "broken" : ""}"><span>${status.frozen ? "!" : "✓"}</span></div><div><span class="eyebrow">INTEGRITY</span><h2>${status.frozen ? "保管庫は凍結されています" : "監査 chain に既知の異常はありません"}</h2><p>意味変更は device ごとの署名 event と、本文を含まない audit hash chain に記録されます。</p></div></section><section class="audit-list"><div><span>書き込み状態</span><strong>${status.frozen ? "FROZEN" : "ACTIVE"}</strong><small>${status.frozen_reason ?? "変更を受け付けられます"}</small></div><div><span>外部モデル</span><strong>OFF</strong><small>自動 fallback なし</small></div><div><span>ネットワーク</span><strong>${escapeHtml(status.network_activity)}</strong><small>telemetry なし</small></div><div><span>復旧状態</span><strong>${status.recovery_exported ? "EXPORTED" : "ACTION REQUIRED"}</strong><small>未確認を安全と表示しません</small></div></section><div class="audit-actions"><button class="secondary-button" data-action="verify-audit">監査 chain を再検証</button><button class="danger-button" data-action="freeze">緊急凍結</button></div>`;
}

function inspector(): string {
  if (state.view === "sources" && state.selectedSource) {
    const source = state.selectedSource;
    return `<aside class="inspector" aria-label="選択した原資料の詳細"><div class="inspector-head"><span>EVIDENCE</span><button aria-label="詳細を閉じる" data-action="close-inspector">×</button></div><h2>${escapeHtml(source.title ?? "無題の原資料")}</h2><p class="inspector-id">${escapeHtml(source.source_id)}</p><dl><div><dt>由来</dt><dd>${escapeHtml(source.provider)}</dd></div><div><dt>期間</dt><dd>${dateLabel(source.occurred_from ?? source.captured_at)}</dd></div><div><dt>機密</dt><dd>${sensitivityLabel(source.sensitivity)}</dd></div><div><dt>完全性</dt><dd>検証済み</dd></div></dl><div class="hash-block"><span>暗号 object</span><code>${escapeHtml(source.integrity)}</code></div><div class="fragment-stack"><span>引用できる断片</span>${state.fragments.length ? state.fragments.slice(0, 6).map(fragmentItem).join("") : "<p>断片を読み込み中…</p>"}</div></aside>`;
  }
  return `<aside class="inspector inspector-calm" aria-label="Pensiveの原則"><span class="calm-mark" aria-hidden="true"></span><blockquote>原資料は、何が残されていたかを記録する。<br />記憶は、それが何を意味し得るかを訂正可能な主張として保持する。</blockquote><small>Source ≠ Memory</small></aside>`;
}

function fragmentItem(fragment: FragmentSummary): string {
  return `<article><span>${escapeHtml(fragment.role ?? "fragment")} · ${dateLabel(fragment.occurred_at)}</span><p>${escapeHtml(fragment.text)}</p>${fragment.injection_candidate ? "<em>指示らしき文を含む — 証拠としてのみ扱う</em>" : ""}${fragment.secret_candidate ? "<em>Secret候補 — 外部送信から除外</em>" : ""}</article>`;
}

function emptyRow(message: string): string {
  return `<div class="empty-row">${escapeHtml(message)}</div>`;
}

function onboarding(): string {
  return `<div class="modal-backdrop"><section class="onboarding-modal" role="dialog" aria-modal="true" aria-labelledby="onboarding-title"><span class="modal-mark" aria-hidden="true"></span><p>PENSIVE MESH</p><h1 id="onboarding-title">自分の記憶を、自分の手元へ。</h1><p class="modal-copy">Pensive は原資料を暗号化し、AIの推論を訂正できる候補として分けて保存します。外部モデルへの通信はありません。</p><div class="mode-switch"><button class="active" data-onboarding-mode="open">保管庫を開く</button><button data-onboarding-mode="create">新しく作る</button></div><label>保管庫の場所<div class="path-input"><input id="vault-path" placeholder="/path/to/PensiveVault" /><button data-action="choose-vault">選ぶ</button></div></label><label>パスフレーズ<input id="vault-passphrase" type="password" autocomplete="current-password" /></label><label id="confirmation-row" class="hidden">パスフレーズを確認<input id="vault-confirmation" type="password" autocomplete="new-password" /></label><button class="primary-button wide" data-action="submit-onboarding">保管庫を開く</button><small>鍵、本文、検索語はログや外部サービスへ送りません。</small></section></div>`;
}

function toast(): string {
  if (!state.toast && !state.error && !state.busy) return "";
  const kind = state.error ? "error" : state.busy ? "busy" : "success";
  const text =
    state.error ??
    (state.busy ? "安全に処理しています…" : (state.toast ?? "完了しました"));
  return `<div class="toast ${kind}" role="status"><i></i>${escapeHtml(text)}</div>`;
}

function wireEvents(): void {
  document.querySelectorAll<HTMLElement>("[data-view]").forEach((element) => {
    element.addEventListener("click", () => {
      const view = element.dataset.view as View;
      if (!view) return;
      state.view = view;
      state.error = null;
      render();
    });
  });
  document.querySelectorAll<HTMLElement>("[data-action]").forEach((element) => {
    element.addEventListener(
      "click",
      () => void action(element.dataset.action ?? "", element),
    );
  });
  document
    .querySelectorAll<HTMLElement>("[data-source-id]")
    .forEach((element) => {
      element.addEventListener(
        "click",
        () => void selectSource(element.dataset.sourceId ?? ""),
      );
    });
  document.querySelectorAll<HTMLElement>("[data-review]").forEach((element) => {
    element.addEventListener(
      "click",
      () =>
        void review(
          element.dataset.memoryId ?? "",
          element.dataset.review ?? "",
        ),
    );
  });
  document
    .querySelectorAll<HTMLElement>("[data-onboarding-mode]")
    .forEach((element) => {
      element.addEventListener("click", () =>
        switchOnboardingMode(element.dataset.onboardingMode === "create"),
      );
    });
  document
    .querySelector<HTMLInputElement>("#source-filter")
    ?.addEventListener("input", filterSources);
  document
    .querySelector<HTMLSelectElement>("#source-sensitivity")
    ?.addEventListener("change", filterSources);
}

let createMode = false;

async function action(name: string, element: HTMLElement): Promise<void> {
  try {
    state.error = null;
    switch (name) {
      case "show-onboarding":
        state.onboarding = true;
        render();
        return;
      case "choose-vault": {
        if (!isTauri) return;
        const selected = await open({ directory: true, multiple: false });
        if (selected) setInput("vault-path", selected);
        return;
      }
      case "submit-onboarding":
        await submitOnboarding();
        return;
      case "lock":
        if (isTauri) await invoke("lock_vault");
        state.status = null;
        state.onboarding = true;
        render();
        return;
      case "import":
        await importSource();
        return;
      case "search":
        await searchVault();
        return;
      case "build-context":
        await buildContext();
        return;
      case "export-context":
        await exportContext(element.dataset.packId ?? "");
        return;
      case "export-recovery":
        await exportRecovery();
        return;
      case "backup":
        await createBackup();
        return;
      case "recovery-test":
        await recoveryTest();
        return;
      case "verify-audit":
        await runBusy(async () => {
          if (isTauri) await invoke("vault_status");
          state.toast = "監査 chain を再検証しました。";
        });
        return;
      case "freeze":
        state.error =
          "緊急凍結は CLI の `pensive freeze` から理由を記録して実行してください。";
        render();
        return;
      case "close-inspector":
        state.selectedSource = null;
        state.fragments = [];
        render();
        return;
    }
  } catch (error) {
    state.error = messageFrom(error);
    state.busy = false;
    render();
  }
}

function switchOnboardingMode(create: boolean): void {
  createMode = create;
  document
    .querySelectorAll("[data-onboarding-mode]")
    .forEach((button) => button.classList.remove("active"));
  document
    .querySelector(`[data-onboarding-mode="${create ? "create" : "open"}"]`)
    ?.classList.add("active");
  document
    .querySelector("#confirmation-row")
    ?.classList.toggle("hidden", !create);
  const submit = document.querySelector<HTMLButtonElement>(
    "[data-action='submit-onboarding']",
  );
  if (submit)
    submit.textContent = create ? "暗号化保管庫を作る" : "保管庫を開く";
}

async function submitOnboarding(): Promise<void> {
  const path = inputValue("vault-path");
  const passphrase = inputValue("vault-passphrase");
  if (!path || !passphrase)
    throw new Error("保管庫の場所とパスフレーズを入力してください。");
  if (createMode && passphrase !== inputValue("vault-confirmation")) {
    throw new Error("確認用パスフレーズが一致しません。");
  }
  if (!isTauri) {
    installDemo();
    state.onboarding = false;
    state.toast = "サンプル保管庫を開きました。";
    render();
    return;
  }
  await runBusy(async () => {
    state.status = await invoke<VaultStatus>(
      createMode ? "init_vault" : "unlock_vault",
      {
        path,
        passphrase,
      },
    );
    state.onboarding = false;
    await refreshAll();
    state.toast = createMode
      ? "暗号化保管庫を作成しました。"
      : "保管庫を開きました。";
  });
}

async function importSource(): Promise<void> {
  if (!isTauri) {
    state.toast = "サンプル表示では実ファイルを読みません。Tauri版で試せます。";
    render();
    return;
  }
  const selected = await open({
    multiple: false,
    filters: [
      { name: "Source", extensions: ["zip", "json", "md", "txt", "csv"] },
    ],
  });
  if (!selected) return;
  await runBusy(async () => {
    const report = await invoke<{
      sources_added: number;
      skipped_duplicates: number;
    }>("import_source", {
      path: selected,
      sensitivity: "SENSITIVE",
    });
    await refreshAll();
    state.toast = `${report.sources_added}件を保存、${report.skipped_duplicates}件の重複を除外しました。`;
  });
}

async function selectSource(sourceId: string): Promise<void> {
  state.selectedSource =
    state.sources.find((source) => source.source_id === sourceId) ?? null;
  state.fragments = isTauri
    ? await invoke<FragmentSummary[]>("list_fragments", { sourceId })
    : demoFragments.filter((fragment) => fragment.source_id === sourceId);
  render();
}

async function review(memoryId: string, reviewAction: string): Promise<void> {
  const memory = state.memories.find((item) => item.memory_id === memoryId);
  if (!memory) return;
  let statement: string | null = null;
  if (reviewAction === "correct") {
    statement = window.prompt(
      "訂正した主張を入力してください。以前の版は履歴に残ります。",
      memory.statement,
    );
    if (!statement) return;
  }
  await runBusy(async () => {
    if (isTauri) {
      await invoke("review_memory", {
        memoryId,
        action: reviewAction,
        statement,
      });
      await refreshAll();
    } else {
      state.memories = state.memories.filter(
        (item) => item.memory_id !== memoryId,
      );
      if (state.status) state.status.memory_inbox_count = state.memories.length;
    }
    state.toast =
      reviewAction === "accept"
        ? "根拠付きの記憶として承認しました。"
        : "判断を履歴へ記録しました。";
  });
}

async function searchVault(): Promise<void> {
  const query = inputValue("ask-query");
  if (!query) throw new Error("問いを入力してください。");
  await runBusy(async () => {
    state.hits = isTauri
      ? await invoke<SearchHit[]>("search_vault", { query })
      : demoFragments.map((fragment) => ({
          fragment,
          source_title: "Pensive の安全境界",
          source_provider: "chatgpt",
          source_state: "ACTIVE",
          rank: 0.92,
          accepted_memories: [],
          contradictions:
            fragment.external_id === "message-branch-a"
              ? ["別の時点の主張と競合"]
              : [],
          why_used:
            "質問語と一致し、出典・時点・branch locator が揃っているため。",
        }));
    state.toast = state.hits.length
      ? `${state.hits.length}件の根拠を見つけました。`
      : "根拠は見つかりませんでした。";
  });
}

async function buildContext(): Promise<void> {
  const purpose = inputValue("context-purpose");
  const query = inputValue("context-query");
  const maxTokens = Number(inputValue("context-budget"));
  if (!purpose || !query)
    throw new Error("目的と検索する文脈を入力してください。");
  await runBusy(async () => {
    state.contextPack = isTauri
      ? await invoke<ContextPack>("build_context_pack", {
          purpose,
          query,
          maxTokens,
        })
      : demoContextPack(purpose, query);
    state.toast =
      "最小必要範囲のプレビューを作りました。まだ外部へ送っていません。";
  });
}

async function exportContext(packId: string): Promise<void> {
  if (!isTauri) {
    state.toast = "サンプル表示では書き出しません。";
    render();
    return;
  }
  const output = await save({
    defaultPath: "pensive-context.pmx",
    filters: [{ name: "Encrypted Context Pack", extensions: ["pmx"] }],
  });
  if (!output) return;
  await runBusy(async () => {
    await invoke("export_context_pack", { packId, output });
    state.toast = "暗号化 Context Pack を書き出しました。";
  });
}

async function exportRecovery(): Promise<void> {
  if (!isTauri) {
    state.toast = "サンプル表示では Recovery Kit を作りません。";
    render();
    return;
  }
  const output = await save({
    defaultPath: "pensive-recovery.pmr",
    filters: [{ name: "Pensive Recovery Kit", extensions: ["pmr"] }],
  });
  if (!output) return;
  const passphrase = window.prompt(
    "Recovery Kit 専用の12文字以上のパスフレーズを入力してください。",
  );
  if (!passphrase) return;
  await runBusy(async () => {
    await invoke("export_recovery_kit", {
      output,
      recoveryPassphrase: passphrase,
    });
    await refreshAll();
    state.toast =
      "暗号化 Recovery Kit を書き出しました。別媒体へ保存してください。";
  });
}

async function createBackup(): Promise<void> {
  if (!isTauri) {
    state.toast = "サンプル表示では Backup を作りません。";
    render();
    return;
  }
  const parent = await open({ directory: true, multiple: false });
  if (!parent) return;
  const output = `${parent}/pensive-backup-${Date.now()}`;
  await runBusy(async () => {
    await invoke("create_backup", { output });
    state.toast = "暗号化 Backup と検証 manifest を作りました。";
  });
}

async function recoveryTest(): Promise<void> {
  if (!isTauri) {
    state.toast = "サンプル表示では clean restore を実行しません。";
    render();
    return;
  }
  const backup = await open({
    directory: true,
    multiple: false,
    title: "Backupフォルダを選択",
  });
  if (!backup) return;
  const kit = await open({
    multiple: false,
    title: "Recovery Kitを選択",
    filters: [{ name: "Pensive Recovery Kit", extensions: ["pmr"] }],
  });
  if (!kit) return;
  const recoveryPassphrase = window.prompt(
    "Recovery Kit のパスフレーズを入力してください。",
  );
  const testUnlockPassphrase = window.prompt(
    "一時復旧環境用の12文字以上のパスフレーズを入力してください。",
  );
  if (!recoveryPassphrase || !testUnlockPassphrase) return;
  await runBusy(async () => {
    await invoke("test_recovery", {
      backup,
      kit,
      recoveryPassphrase,
      testUnlockPassphrase,
    });
    await refreshAll();
    state.toast = "Clean restore、hash、audit chain の検証に成功しました。";
  });
}

async function runBusy(operation: () => Promise<void>): Promise<void> {
  state.busy = true;
  state.toast = null;
  state.error = null;
  render();
  try {
    await operation();
  } finally {
    state.busy = false;
    render();
  }
}

function demoContextPack(purpose: string, query: string): ContextPack {
  return {
    protocol: "pensive-context-pack/1",
    pack_id: "pack-demo-01",
    purpose,
    query,
    created_at: new Date().toISOString(),
    expires_at: new Date(Date.now() + 86_400_000).toISOString(),
    summary: "2件の原資料断片と1件の訂正可能な記憶を選びました。",
    memory_items: state.memories.slice(0, 1),
    contradictions: ["別時点の主張が1件あります。"],
    source_fragments: demoFragments,
    omissions: ["Secret候補 1件を既定ポリシーで除外"],
    redactions: [],
    integrity: {
      canonical_digest: "blake3:9c612d4e87d3b82e6d34c6f4a013b1c8",
      signed_by_device: "sample-device",
      signature: "sample-signature",
    },
  };
}

function filterSources(): void {
  const query = inputValue("source-filter").toLocaleLowerCase("ja");
  const sensitivity = inputValue("source-sensitivity");
  document.querySelectorAll<HTMLElement>("[data-source-id]").forEach((row) => {
    const source = state.sources.find(
      (item) => item.source_id === row.dataset.sourceId,
    );
    const matchesQuery =
      !query ||
      `${source?.title ?? ""} ${source?.provider ?? ""}`
        .toLocaleLowerCase("ja")
        .includes(query);
    const matchesSensitivity =
      sensitivity === "all" || source?.sensitivity === sensitivity;
    row.hidden = !(matchesQuery && matchesSensitivity);
  });
}

function inputValue(id: string): string {
  const input = document.querySelector<
    HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
  >(`#${id}`);
  return input?.value.trim() ?? "";
}

function setInput(id: string, value: string): void {
  const input = document.querySelector<HTMLInputElement>(`#${id}`);
  if (input) input.value = value;
}

function sensitivityLabel(value: string): string {
  const labels: Record<string, string> = {
    PERSONAL: "個人",
    SENSITIVE: "機密",
    HIGHLY_SENSITIVE: "高機密",
    SECRET: "秘密",
  };
  return labels[value] ?? value;
}

function dateLabel(value: string | null): string {
  if (!value) return "時点なし";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("ja-JP", {
    year: "numeric",
    month: "short",
    day: "numeric",
  }).format(date);
}

function escapeHtml(value: string): string {
  return value.replace(
    /[&<>'"]/g,
    (character) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        "'": "&#39;",
        '"': "&quot;",
      })[character] ?? character,
  );
}

function escapeAttr(value: string): string {
  return escapeHtml(value);
}

function messageFrom(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

void start();
