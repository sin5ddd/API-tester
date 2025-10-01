![API Tester Icon](../assets/api-tester-icon.jpg)

# API Tester (Rust + egui)

軽量・高速なネイティブ GUI で REST API をテスト・可視化するツールです。巨大ではない（〜1MB）JSON を前提に `serde` で扱い、ツリー表示・インライン差分・クエリ（JSONPath/JMESPath）など、開発時の検証に便利な機能を詰め込んでいます。

本リポジトリは Git のタグ（例: `v0.1.0`）を push すると GitHub Actions が各 OS 向けバイナリを自動ビルドし、Release に添付します。

## 主な機能
- HTTP メソッド: GET / POST / PUT / PATCH / DELETE
- 背景スレッドでの送信（UI フリーズ防止）
- 認証: None / Basic / Bearer / OAuth2 Token
- ヘッダ編集（1 行 1 ヘッダ: `Key: Value` 形式）
- リクエスト Body:
  - JSON 入力（整形表示）
  - Form（key=value）エディタ（有効/無効、削除、+Add）
  - JSON と Form の双方向同期（常に一致）
- レスポンス表示:
  - ツリー表示（オブジェクト/配列を折り畳み）
  - スクロール可能な領域
  - ステータス表示の色分け（200=緑、3xx/その他=黄、4xx/5xx=赤）
- 差分（必須要件）:
  - 構造的 diff（`json_patch::diff`）をインラインでツリーに反映
  - 置換ノードは現行値の隣に旧値（グレー）も表示
- クエリ（常時ツリーに反映）:
  - JSONPath（抽出）
  - JMESPath（抽出＋整形）
  - Run すると結果がツリーにフィルタ適用され、そのまま展開・閲覧
- 検索（プレーンテキスト）: 整形 JSON の該当行を表示
- プロファイル管理（プロジェクト/フォルダ毎）:
  - 保存場所: ユーザーホームディレクトリ（Windows: `%USERPROFILE%\.api-tester\profiles\`、Mac/Linux: `~/.api-tester/profiles/`）
  - フォルダ風エクスプローラ UI（プロジェクト=フォルダ、プロファイル=ファイル）
  - 新規/保存/上書き/読み込み/削除/リネーム（プロジェクト・プロファイル）
- cURL インポート: method/url/headers/body を簡易解析して適用
- エクスポート: JSON、NDJSON（配列）、CSV（配列をフラット化）
- 自動ポーリング: 間隔指定で送信、差分はインライン反映
- ウィンドウ「常に手前に表示」（Float/Unfloat 切替）
- コンパクト UI（行間・余白・フォントを詰め気味に）

## スクリーンショット
![screenshot](../assets/screenshot.png)

## 使い方
1. URL とメソッドを設定
2. 必要に応じて Auth/Headers/Body を編集
   - Body は JSON と Form（key=value）が切り替え可能。双方向同期で常に同じ内容になります
3. Send を押すと背景スレッドで送信
4. Tree で JSON を確認（インライン差分・旧値表示が自動適用）
5. Query の JSONPath/JMESPath を Run すると、Tree が結果でフィルタ表示されます
6. Profiles でプロジェクト/プロファイルを管理（ダブルクリックで読み込み）
7. Export で JSON/NDJSON/CSV を保存

## インストール
- GitHub Releases から OS に合ったアーカイブをダウンロード・展開してください。
  - 例: `api-tester-v0.1.0-windows-x86_64.zip` (Windows/macOS)、`api-tester-v0.1.0-linux-x86_64.tar.gz` (Linux)
- Windows では MSVC ランタイムが必要になる場合があります。起動しない場合は Visual C++ 再頒布可能パッケージの導入を検討してください。

## 自動リリース（GitHub Actions + タグ）
本リポジトリには `.github/workflows/release.yml` が含まれており、`v*` 形式のタグが push されると CI が起動します。

手順:
```
# 変更をコミット
git add -A
git commit -m "chore: prepare v0.1.0"

# タグ作成（例）
git tag v0.1.0

# タグをリモートへ push
git push origin v0.1.0
# あるいは
# git push origin --tags
```
CI の流れ:
- Windows / Linux / macOS でリリースビルド
- バイナリ（`API-tester` / `API-tester.exe` など）と同梱ファイル（README / LICENSE / docs/sample.json があれば）をパッケージ化
- アーカイブ（Windows/macOS: `.zip`、Linux: `.tar.gz`）と SHA256 チェックサムを作成
- 対応する GitHub Release にアセットを自動添付

注: Cargo.toml のパッケージ名は `API-tester` ですが、ワークフローは大小文字/ハイフン差異を考慮してバイナリ名を検出するようになっています。

## ソースからビルド
Rust の stable がインストール済みであることを確認し、以下を実行:
```
cargo build --release
```
生成物:
- Windows: `target\release\API-tester.exe`
- Linux/macOS: `target/release/API-tester`

## 開発
- GUI: eframe/egui（ネイティブ）
- HTTP: reqwest（blocking）
- JSON: serde/serde_json
- Diff: json-patch（構造差分 必須）、similar（テキスト差分）
- Query: jsonpath_lib / jmespath
- その他: rfd（ファイルダイアログ）、url、base64、csv、anyhow

## ロードマップ
- 削除差分（remove）もツリー上で参照できるヒント UI の導入
- 巨大配列の仮想スクロール/ページング
- 認証フローの追加（OAuth2 Code Flow 等）

## ライセンス
未定（必要に応じて `LICENSE` を追加してください）。

## 変更履歴
詳しくは [CHANGELOG.md](CHANGELOG.md) を参照してください。
