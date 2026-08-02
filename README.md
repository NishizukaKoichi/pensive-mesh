# Pensive Mesh

> 自分の記憶を、自分が所有できる形へ戻す。

Pensive Mesh は、本人が選んだ原資料をローカルで暗号化し、そこから得た「記憶」を出典・時点・不確実性を持つ訂正可能な主張として分けて扱う、ローカルファーストの人格文脈基盤です。

このリポジトリは **v0.1 Local Memory Core** の動作実装です。ChatGPT export をモデルや API key なしで取り込み、原資料・会話 branch・証拠を閲覧し、記憶候補を本人が確認し、必要最小限の Context Pack を暗号化して書き出せます。

## 現在できること

- ランダムな Vault Root Key とパスフレーズ用 Argon2id key envelope
- SQLCipher で catalog、FTS、Memory、監査情報を全面暗号化
- XChaCha20-Poly1305 で Source object と Journal event を個別暗号化
- Ed25519 device key による append-only Memory Event 署名
- ChatGPT `conversations.json` / ZIP の逐次・増分取込
- 同一 export の重複排除、部分更新の新 revision 化
- message parent、全 branch、timestamp、attachment の保持
- prompt injection / Secret候補の識別（命令としては実行しない）
- 証拠必須の Memory candidate、承認・訂正・却下・失効
- 矛盾を消さない Conflict Record
- 暗号化 FTS 上のローカル検索と「なぜ使ったか」の表示
- Secret・第三者情報を既定除外する署名済み Context Pack
- Recovery Kit、整合 Backup、clean-room restore 試験
- Rust CLI と Tauri 2 desktop UI
- 外部モデル、telemetry、URL fetch、メール・予定・購入・公開・shell 実行コードなし

## 信頼境界

```text
explicit Source import
        ↓
encrypted immutable Source ──→ local evidence search
        ↓
revisable Memory candidate ──→ user review
        ↓
minimal signed Context Pack
        ↓
   (v0.1 ends here)

future action candidate → SimWorld → DRAFT Spell Ticket
                                      ↓
                            separate Spell Kernel only

encrypted objects/journals → Arcane Adapter only
```

Pensive 自身は外部世界を変更しません。Spell Kernel や Arcane Commons Mesh の DB も読み書きしません。公開された schema、署名 file、typed adapter だけが境界です。

## 必要環境

- macOS 13 以降（v0.1 の配布・実利用対象）
- Rust 1.88（`rust-toolchain.toml` が固定）
- Node.js 22（`.nvmrc` が固定）
- pnpm 10.13.1
- Xcode Command Line Tools

Windows / Linux の core build は CI 対象ですが、v0.1 の desktop package は macOS で検証します。

## セットアップ

```bash
git clone https://github.com/NishizukaKoichi/pensive-mesh.git
cd pensive-mesh
pnpm install --frozen-lockfile
pnpm typecheck
pnpm test
pnpm build
```

Desktop preview（サンプル表示、実データを読みません）:

```bash
pnpm dev
```

Desktop app:

```bash
pnpm desktop:dev
```

macOS app bundle:

```bash
pnpm desktop:build
open target/release/bundle/macos/Pensive\ Mesh.app
```

署名・notarization は v0.1 の公開 source build には含みません。macOS の Gatekeeper 警告を回避した正式配布には、所有者の Apple Developer identity と release approval が必要です。

## CLI で最初の保管庫を作る

CLI は秘密を command line、環境変数、shell history に置かず、TTY でパスフレーズを尋ねます。

```bash
cargo run -p pensive -- init /path/to/PensiveVault

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  import chatgpt /path/to/chatgpt-export.zip

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  source list

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  query "Pensiveについて何を決めた？"
```

候補記憶は、検索結果にある `fragment_id` を証拠として指定します。

```bash
cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  memory propose \
  "原資料は要約より強い" \
  --memory-type DECISION \
  --evidence <fragment-id>

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  memory inbox

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  memory accept <memory-id>
```

Context Pack は生成後に自動変化せず、既定24時間で失効します。

```bash
cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  context build \
  "Pensiveの安全境界" \
  --purpose "別のAIへ最小限の背景を渡す"

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  context export <pack-id> \
  --output /safe/place/pensive-context.pmx
```

## Recovery Kit と clean restore

Recovery Kit だけでは Source を復元できません。暗号化 Backup と専用パスフレーズの両方が必要です。通常の保管庫と別の媒体へ保存してください。

```bash
cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  recovery export /external-media/pensive-recovery.pmr

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  backup run /external-media/pensive-backup-20260802

cargo run -p pensive -- \
  --vault /path/to/PensiveVault \
  recovery test \
  --backup /external-media/pensive-backup-20260802 \
  --kit /external-media/pensive-recovery.pmr
```

別の環境へ復元:

```bash
cargo run -p pensive -- recovery restore \
  --backup /external-media/pensive-backup-20260802 \
  --kit /external-media/pensive-recovery.pmr \
  --destination /new/place/PensiveVault
```

既存の空でない destination は上書きしません。詳細は [Recovery runbook](docs/RECOVERY.md) を参照してください。

## 検証

```bash
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm test:integration
pnpm build
pnpm verify:mvp
pnpm scan:plaintext
cargo audit
```

`scan:plaintext` は一時 Vault に固有 marker を保存し、SQLCipher catalog、object、journal、metadata を含む Vault 配下の全ファイルを byte scan します。marker が一つでも平文で残れば失敗します。

## データと秘密

Vault の既定構造:

```text
PensiveVault/
├── vault.json             # format/version/opaque vault id only
├── key-envelope.pmk       # Argon2id + XChaCha20 encrypted Vault Root Key
├── catalog.sqlite         # SQLCipher
├── objects/<prefix>/*.pmo # encrypted Source / attachment objects
├── journals/<device>/*.pmj
├── checkpoints/
├── recovery-status.json   # secretsなし
└── diagnostics/           # content-free only
```

- API key、OAuth token、Passkey、Recovery passphrase は保存しません。
- 外部モデルは v0.1 では実装も有効化もされません。
- runtime Vault は固定 path を要求せず、本人が保存先を選びます。
- unlock 中に OS が完全侵害された場合の完全防御は保証できません。
- revoked device が既に取得した過去平文を遠隔消去することはできません。

## 文書

- [正本プロダクト・システム仕様](docs/PRODUCT_SPEC.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Threat Model](docs/THREAT_MODEL.md)
- [Privacy](docs/PRIVACY.md)
- [Context Pack Protocol](docs/CONTEXT_PACK.md)
- [Recovery runbook](docs/RECOVERY.md)
- [Deletion limits](docs/DELETION.md)
- [Execution Plan](docs/EXECUTION_PLAN.md)
- [Verification Report](docs/VERIFICATION_REPORT.md)
- [Roadmap](docs/ROADMAP.md)

## Version の正直な現在地

v0.1 の対象範囲は local memory core です。v1.0 を名乗るには、少なくとも二つの OS での実利用、二物理端末 sync、地理的 Arcane restore、外部 security review、第三者 portable reader の実証が必要です。これらを README の予定だけで「完成」とは扱いません。

## License

Pensive Mesh のコードとリポジトリ内文書は、個別に別条件が明記されているものを除き [Apache License 2.0](LICENSE) で公開します。利用・改変・再配布の際はライセンス条件を確認してください。選定理由は [ADR 0002](docs/adr/0002-license-decision-required.md) に記録しています。
